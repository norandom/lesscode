"""Collect finite paired observations and frozen native results for NS01."""
from datetime import datetime, timezone
import difflib
import json
from pathlib import Path
import shutil
import zipfile

import conduct
from conduct import ROOT, RUN, EXT, HERE, MODULE, sha, dump, execute, uv

REVIEWED = EXT/'parallel-trial-v3'
conduct.TRIAL = REVIEWED
CONTROLLER_CHANGES = {
    '.kiro/specs/capability-compression-core/tasks.md': 'Controller task7.1 completion commit3f03b52',
    'pyproject.toml': 'User requested pytest-xdist/nextest test tooling',
    'requirements-test.lock': 'User requested pinned test-only pytest-xdist dependencies; new after snapshot',
    'tools/ci_validate.py': 'User requested parallel test tooling',
    '.github/workflows/baseline-platforms.yml': 'User requested Python parallel-test CI',
    '.github/workflows/native-core.yml': 'User requested Rust nextest CI',
    'native/README.md': 'User requested nextest instructions',
    'docs/README.md': 'User requested test-tooling documentation index',
    'docs/testing.md': 'User requested test-tooling documentation; new after snapshot',
}


def adverse():
    for side in ('baseline', 'candidate'):
        tree = EXT/('parallel-v3-adverse-'+side)
        shutil.copytree(RUN/side/'src', tree/'src')
        relative = MODULE if side == 'baseline' else 'src/lesscode_baseline/contracts.py'
        path = tree/relative
        original = path.read_text()
        assert 'if key in seen:' in original
        path.write_text(original.replace('if key in seen:', 'if False and key in seen:', 1), encoding='utf-8')
        result = execute('parallel-v3-'+side+'-adverse-mutant', uv(str(REVIEWED/'expected_runner.py')), ROOT, tree)
        raw = (HERE/('parallel-v3-'+side+'-adverse-mutant.log')).read_text()
        assert result['exit_code'] == 1 and 'FAILED (failures=4)' in raw and 'ContractError not raised' in raw
        dump(HERE/('parallel-v3-'+side+'-adverse.json'), dict(exit_code=0,
            conclusion='Expected duplicate-guard deletion detected by four independent checks',
            source_side=side, frozen_file_sha256=sha(REVIEWED/'frozen.json'),
            mutation_file=relative, expected_mutant_exit=1, observed_mutant_exit=result['exit_code'],
            raw_log_sha256=result['log_sha256']))


def integrity():
    inventory = json.loads((ROOT/'docs/experiments/native-self-apply/source-inventory.json').read_text())
    source_changes = {p:dict(expected=h, observed=sha(ROOT/p) if (ROOT/p).is_file() else None)
                      for p,h in inventory.items() if not (ROOT/p).is_file() or sha(ROOT/p) != h}
    baseline_changes = {p:dict(expected=h, observed=sha(RUN/'baseline'/p))
                        for p,h in inventory.items() if sha(RUN/'baseline'/p) != h}
    candidate_changes = {p:dict(expected=h, observed=sha(RUN/'candidate'/p))
                         for p,h in inventory.items() if sha(RUN/'candidate'/p) != h}
    definition = json.loads((REVIEWED/'check-definition.json').read_text())
    test_changes = {}
    tools = json.loads((ROOT/'docs/experiments/native-self-apply/tool-inventory.json').read_text())
    tool_changes = {}
    worker_provenance = {}
    expected_ids=json.loads((REVIEWED/'baseline-preflight-origin/controller-collection.json').read_text())['nodeids']
    for side in ('baseline','candidate'):
        tree = EXT/'check-runs'/('parallel-v3-'+side)
        changes = {p:dict(expected=h,observed=sha(tree/p)) for p,h in definition['frozen_test_files'].items() if sha(tree/p) != h}
        test_changes[side] = changes
        assert not changes
        tool_changes[side] = {p:dict(expected=h,observed=sha(tree/'.tools/analyzers'/p))
                              for p,h in tools.items() if sha(tree/'.tools/analyzers'/p)!=h}
        assert not tool_changes[side]
        assert sha(tree/'conftest.py')==sha(REVIEWED/'source_guard.py')
        assert sha(tree/'pyproject.toml')==sha(RUN/'baseline/pyproject.toml')
        assert sha(tree/'.gitattributes')==sha(RUN/'baseline/.gitattributes')
        origin=HERE/('parallel-v3-'+side+'-origin')
        origins=sorted(origin.glob('*-origin.json'))
        assert {p.name for p in origins}=={'controller-origin.json',*(f'gw{i}-origin.json' for i in range(4))}
        for path in origins:
            data=json.loads(path.read_text())
            assert Path(data['expected_src'])==(tree/'src')
            for relative,observed in data['observed'].items():
                assert observed['sha256']==sha(RUN/side/'src'/relative)
        collections=sorted(origin.glob('*-collection.json'))
        assert {p.name for p in collections}=={f'gw{i}-collection.json' for i in range(4)}
        for path in collections:
            assert json.loads(path.read_text())['nodeids']==expected_ids
        worker_provenance[side]=dict(guarded_processes=len(origins),matching_worker_collections=len(collections),test_identities=len(expected_ids))
        for check in ('relevant', 'regressions', 'independent', 'import'):
            data = json.loads((HERE/f'parallel-v3-{side}-{check}.json').read_text())
            assert data['exit_code']==0 and data['frozen_file_sha256']==sha(REVIEWED/'frozen.json')
            assert data['source_hashes_before'] == data['source_hashes_after']
            for p,h in data['source_hashes_before'].items():
                assert sha(RUN/side/p) == h
    receipt = json.loads((ROOT/'docs/experiments/native-self-apply/receipt.json').read_text())
    evaluator_unchanged = sha(EXT/'lesscode-core.exe') == receipt['evaluator']['sha256']
    evaluator_source_changes = {p:dict(expected=h,observed=sha(ROOT/p)) for p,h in receipt['evaluator']['source_identity'].items() if sha(ROOT/p)!=h}
    assert evaluator_unchanged and set(evaluator_source_changes) <= {'native/README.md'} and not baseline_changes
    assert set(candidate_changes) == {MODULE}
    assert set(source_changes) <= CONTROLLER_CHANGES.keys()
    live_tool_changes = {p:dict(expected=h,observed=sha(ROOT/'.tools/analyzers'/p))
                         for p,h in tools.items() if sha(ROOT/'.tools/analyzers'/p)!=h}
    assert not live_tool_changes
    result = dict(time=datetime.now(timezone.utc).isoformat(), source_files=len(inventory),
        original_snapshot_deltas=source_changes,
        original_delta_authorization=CONTROLLER_CHANGES,
        new_controller_paths={p:sha(ROOT/p) for p in CONTROLLER_CHANGES if p not in inventory and (ROOT/p).is_file()},
        baseline_snapshot_deltas=baseline_changes, candidate_snapshot_deltas=candidate_changes,
        frozen_test_deltas=test_changes, frozen_tool_deltas=tool_changes, live_tool_deltas=live_tool_changes,
        guarded_worker_provenance=worker_provenance,
        evaluator_unchanged=evaluator_unchanged,
        evaluator_source_deltas=evaluator_source_changes,
        pre_edit_runner_unchanged=sha(REVIEWED/'expected_runner.py')==sha(EXT/'trial/expected_runner.py'),
        pre_edit_independent_runner_unchanged=sha(REVIEWED/'expected_runner.py')==sha(EXT/'trial/expected_runner.py'),
        parallel_commands_changed_with_explicit_user_authorization=True)
    dump(HERE/'integrity.json', result)


def zip_outputs(name, directory):
    with zipfile.ZipFile(HERE/name, 'w', zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(directory.rglob('*')):
            if path.is_file():
                archive.write(path, path.relative_to(directory))


def preliminary_archive():
    with zipfile.ZipFile(HERE/'preliminary-freezes.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
        for phase in ('trial', 'reviewed-trial', 'reviewed-trial-v2', 'parallel-trial', 'parallel-trial-v2'):
            for name in ('manifest.json', 'frozen.json', 'check-definition.json'):
                archive.write(EXT/phase/name, phase+'/'+name)
    dump(HERE/'preliminary-dispositions.json', dict(
        original_trial='7.1 preparation retained unchanged outside this directory; no behavior/compression verdict',
        initial_runner_trial='Checks frozen before first candidate edit; raw mutant RED and candidate flag stages retained',
        candidate_v1='Relevant suite171:7errors from accidental removal of still-used re import; archived source and patch; no adoption',
        candidate_repair='Restored re import only; same reuse hypothesis, fixed target and boundary; candidate v2 retained for independent review',
        terminated_full_commands=[
            dict(phase='baseline/candidate initial', conductor_pid=44240, outcome='terminated before completion; no regression result or pass claimed'),
            dict(phase='candidate-v2 preliminary', conductor_pid=30324, outcome='terminated before completion; no regression result or pass claimed'),
            dict(phase='first reviewed freeze', conductor_pid=23084, outcome='terminated before completion to add explicit property/classmethod descriptor call review before final paired executions')],
        raw_red_note='Flag-OFF import gate is protocol scaffolding; four meaningful expected-error failures of guard-deletion mutant are the substantive pre-edit RED evidence.',
        frozen_runner_limit='No separate empty-string literal in independent runner; whitespace/NUL/non-string cases and unchanged existing tests cover the larger input boundary. Arbitrary custom subclasses are not proved.'))


def measurement_probe():
    # Native report supplies the canonical frozen digest/source identity. It is a
    # source measurement probe only; no outcome from the candidate is an oracle.
    probe = EXT/'parallel-v3-measurement-probe'
    execute('parallel-v3-measurement-probe', [str(EXT/'lesscode-core.exe'), 'compare',
        '--frozen', str(REVIEWED/'frozen.json'), '--baseline', str(RUN/'baseline'),
        '--candidate', str(RUN/'candidate'), '--evidence', str(EXT/'measurement-only-evidence.json'),
        '--artifact-root', str(REVIEWED), '--out', str(probe)], ROOT)
    return json.loads((probe/'result.json').read_text())['comparison']


def evidence():
    probe = EXT/'parallel-v3-measurement-probe'
    result = json.loads((probe/'result.json').read_text())['comparison'] if (probe/'result.json').exists() else measurement_probe()
    manifest = json.loads((REVIEWED/'manifest.json').read_text())
    ownership = json.loads((EXT/'measurement-only-evidence.json').read_text())['candidate_ownership']
    ownership['review'] = json.loads((HERE/'candidate-scope-review.json').read_text())
    checks = {}
    for check, suffix in [('relevant', 'relevant'), ('project_regressions', 'regressions'),
                          ('independent_expected', 'independent'), ('adverse', 'adverse')]:
        pair = dict(schema_version=1)
        for side in ('baseline','candidate'):
            observation = json.loads((HERE/f'parallel-v3-{side}-{suffix}.json').read_text())
            assert observation['exit_code'] == 0
            names = [f'parallel-v3-{side}-{suffix}.json']
            names += [f'parallel-v3-{side}-adverse-mutant.log'] if check == 'adverse' else [f'parallel-v3-{side}-{suffix}.log']
            if check == 'project_regressions':
                names += [f'parallel-v3-{side}-junit.xml', f'parallel-v3-{side}-preflight.json',
                          'source_guard.py','parallel-v3-check-definition.json','parallel-v3-requirements-test.lock','integrity.json']
                for path in (HERE/f'parallel-v3-{side}-origin').iterdir():
                    names.append(path.relative_to(HERE).as_posix())
            artifacts = {name:dict(schema_version=1,path=name,sha256=sha(HERE/name)) for name in names}
            pair[side] = dict(schema_version=1, source_identity=result[side+'_identity'],
                frozen_trial_digest=result['frozen_trial_digest'], definition_digest=manifest['checks'][check]['definition_digest'],
                status='passed', artifacts=artifacts, summary=dict(kind='exit_status',code=0),
                tested_domain={'relevant':'171 unchanged contract, rule-contract, evidence and rule-evaluation tests',
                    'project_regressions':'All515 existing unittest identities via pinned pytest9.1.1/xdist3.8.0, four loadfile workers, Windows and frozen analyzer installation; JUnit retained',
                    'independent_expected':'24 frozen JSON obligations plus seven hand-authored test methods for types, exact errors, order, keywords, numeric/path/time boundaries and typed serialization',
                    'adverse':'Guard-deletion mutant detected at repeated identity index1; malformed fingerprint/unknown-field invalid inputs rejected'}[check],
                limitations=['Finite checks on CPython3.12.10/Windows; arbitrary hostile subclasses and all Python objects not proved; no Rust self-compression claim.'])
        checks[check] = pair
    bundle = dict(schema_version=1,candidate_ownership=ownership,checks=checks,dependency_changes={})
    dump(HERE/'evidence.json',bundle)
    final = EXT/'parallel-v3-comparison'
    for suffix in ('json','log'):
        previous=HERE/('parallel-v3-compare.'+suffix)
        if previous.exists():
            shutil.copyfile(previous,HERE/('initial-admission-invocation.'+suffix))
    execute('parallel-v3-compare', [str(EXT/'lesscode-core.exe'), 'compare', '--frozen', str(REVIEWED/'frozen.json'),
        '--baseline', str(RUN/'baseline'), '--candidate', str(RUN/'candidate'), '--evidence', str(HERE/'evidence.json'),
        '--artifact-root', str(HERE), '--out', str(final)], ROOT)
    output = json.loads((final/'result.json').read_text())['comparison']
    summary={key:output[key] for key in ('trial_id','baseline_identity','candidate_identity',
        'frozen_trial_digest','status','reasons','check_findings')}
    summary['limitations']='Finite Windows/CPython checks; dynamic call layers unavailable; arbitrary hostile objects and Rust/TypeScript self-compression unverified. Full unchanged native limitation catalog is in native-result.zip.'
    dump(HERE/'result-summary.json',summary)
    dump(HERE/'counts.json', {key:output['accounting'][key] for key in ('capability_metrics','wider_metrics','category_totals')})
    zip_outputs('native-result.zip', final)
    zip_outputs('incomplete-preliminary-native.zip', EXT/'trial/count-probe')
    patch = ''.join(difflib.unified_diff((RUN/'baseline'/MODULE).read_text().splitlines(True),
        (RUN/'candidate'/MODULE).read_text().splitlines(True),fromfile='a/'+MODULE,tofile='b/'+MODULE))
    (HERE/'candidate.patch').write_text(patch,encoding='utf-8')
    with zipfile.ZipFile(HERE/'candidate-source.zip','w',zipfile.ZIP_DEFLATED) as archive:
        archive.write(RUN/'candidate'/MODULE,MODULE)
    preliminary_archive()
    dump(HERE/'durable-artifacts.json', {p.name:sha(p) for p in HERE.iterdir() if p.is_file() and p.name!='durable-artifacts.json'})


if __name__ == '__main__':
    import sys
    {'adverse':adverse,'integrity':integrity,'evidence':evidence,'probe':measurement_probe}[sys.argv[1]]()
