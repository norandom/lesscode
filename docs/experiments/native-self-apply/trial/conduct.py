"""Reproducible NS01 trial apparatus; support is outside measured source."""
import ast
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path('D:/Source/lesscode')
RUN = ROOT / '.runs/native-self-apply/ns01'
EXT = RUN / 'external'
HERE = Path(__file__).resolve().parent
TRIAL = EXT / 'trial'
PYTHON = ROOT / '.venv-task11/Scripts/python.exe'
MODULE = 'src/lesscode_baseline/rules/contracts.py'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def dump(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def execute(name, command, cwd, source=None, timeout=1800, extra_env=None):
    env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
    if source:
        env['PYTHONPATH'] = str(source / 'src')
    if extra_env:
        env.update(extra_env)
    source_before = {str(p.relative_to(source)): sha(p) for p in (source/'src').rglob('*') if p.is_file()} if source else {}
    start = time.monotonic()
    with (HERE / (name + '.log')).open('wb') as output:
        result = subprocess.run(command, cwd=cwd, env=env, stdout=output,
                                stderr=subprocess.STDOUT, timeout=timeout)
    source_after = {str(p.relative_to(source)): sha(p) for p in (source/'src').rglob('*') if p.is_file()} if source else {}
    metadata = dict(command=[str(x) for x in command], cwd=str(cwd),
                    PYTHONPATH=env.get('PYTHONPATH'), exit_code=result.returncode,
                    elapsed_seconds=time.monotonic()-start, log_sha256=sha(HERE / (name+'.log')),
                    frozen_file_sha256=sha(TRIAL/'frozen.json') if (TRIAL/'frozen.json').exists() else None,
                    source_hashes_before=source_before, source_hashes_after=source_after)
    dump(HERE / (name + '.json'), metadata)
    print(name, result.returncode, metadata['elapsed_seconds'], flush=True)
    return metadata


def uv(*args):
    return ['uv.exe', 'run', '--no-project', '--no-python-downloads', '--python', str(PYTHON), 'python', *args]


def freeze():
    TRIAL.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(ROOT / 'docs/experiments/native-self-apply/expected-outcomes.json', HERE / 'expected-outcomes.json')
    for filename in ('expected_runner.py', 'expected-outcomes.json'):
        shutil.copyfile(HERE / filename, TRIAL / filename)
    definition = json.loads((EXT/'check-definition.json').read_text())
    definition['extension'] = dict(runner_sha256=sha(TRIAL/'expected_runner.py'),
        corpus_sha256=sha(TRIAL/'expected-outcomes.json'),
        provenance='Hand-derived expectations frozen before candidate edit; original definitions untouched',
        runner_command=['python', str(TRIAL/'expected_runner.py')],
        adverse_command='Same runner after duplicate-check removal in separate adverse source tree')
    dump(TRIAL/'check-definition.json', definition)
    manifest = json.loads((EXT/'manifest.json').read_text())
    manifest['trial_id'] += '-independent-v1'
    for check in manifest['checks'].values():
        check['definition_digest'] = sha(TRIAL/'check-definition.json')
    dump(TRIAL/'manifest.json', manifest)
    shutil.copyfile(EXT/'inventory.json', TRIAL/'inventory.json')
    execute('prepare', [str(EXT/'lesscode-core.exe'), 'prepare', '--manifest', str(TRIAL/'manifest.json'),
        '--baseline', str(RUN/'baseline'), '--artifact-root', str(TRIAL), '--out', str(TRIAL/'frozen.json')], ROOT)
    for path in TRIAL.iterdir():
        if path.is_file():
            shutil.copyfile(path, HERE/path.name)
    dump(HERE/'preregistration.json', dict(time=datetime.now(timezone.utc).isoformat(),
        candidate_unchanged=sha(RUN/'candidate'/MODULE)==sha(RUN/'baseline'/MODULE),
        runner_sha256=sha(TRIAL/'expected_runner.py'), frozen_sha256=sha(TRIAL/'frozen.json'),
        target='25 native statements; no physical/decision/function increases', attempt_limit=1))


def independent(side, suffix=''):
    return execute(side+'-independent'+suffix, uv(str(TRIAL/'expected_runner.py')), ROOT, RUN/side)


def red():
    adverse = RUN/'adverse'
    shutil.copytree(RUN/'baseline'/'src', adverse/'src')
    path = adverse/MODULE
    source = path.read_text()
    source = source.replace('if key in seen:', 'if False and key in seen:', 1)
    path.write_text(source, encoding='utf-8')
    result = independent('adverse', '-mutant')
    assert result['exit_code'] != 0


def candidate():
    path = RUN/'candidate'/MODULE
    source = path.read_text()
    tree = ast.parse(source)
    lines = source.splitlines(keepends=True)
    replacements = {
        '_fail': 'def _fail(path: str, reason: str):\n    return _baseline_fail(path, reason)\n',
        '_object': 'def _object(value, path: str, fields: str) -> dict:\n    return _baseline_object(value, path, fields)\n',
        '_choice': 'def _choice(value, path: str, choices: tuple[str, ...]) -> str:\n    return _baseline_choice(value, path, choices)\n',
        '_string': '', '_hash': '', '_sequence': '',
    }
    for node in reversed(tree.body):
        if isinstance(node, ast.FunctionDef) and node.name in replacements:
            lines[node.lineno-1:node.end_lineno] = [replacements[node.name]]
    source = ''.join(lines).replace('import re\n', '')
    source = source.replace('from lesscode_baseline.snapshot import matches',
        'from lesscode_baseline.contracts import (\n    _fail as _baseline_fail, _object as _baseline_object, _choice as _baseline_choice,\n    _string, _hash, _sequence,\n)\nfrom lesscode_baseline.snapshot import matches')
    while '\n\n\n\n' in source:
        source = source.replace('\n\n\n\n', '\n\n\n')
    # Temporary flag: off preserves baseline; on selects reuse. Removed after green.
    original = path.read_bytes()
    flagged = source + '\n_NS01_REUSE_ENABLED = False\nif not _NS01_REUSE_ENABLED:\n    raise RuntimeError("NS01 candidate reuse flag OFF")\n'
    path.write_text(flagged, encoding='utf-8')
    assert independent('candidate', '-flag-off')['exit_code'] != 0
    path.write_text(flagged.replace('_NS01_REUSE_ENABLED = False', '_NS01_REUSE_ENABLED = True'), encoding='utf-8')
    assert independent('candidate', '-flag-on')['exit_code'] == 0
    path.write_text(source, encoding='utf-8')
    assert independent('candidate')['exit_code'] == 0
    patch = ''.join(difflib.unified_diff(original.decode().splitlines(True), source.splitlines(True),
        fromfile='a/'+MODULE, tofile='b/'+MODULE))
    (HERE/'candidate.patch').write_text(patch, encoding='utf-8')
    shutil.copyfile(path, HERE/'candidate-contracts.py')


def regressions(side, source_side=None):
    tree = EXT/'check-runs'/side
    shutil.copytree(EXT/'frozen-checks', tree)
    shutil.copytree(RUN/(source_side or side)/'src', tree/'src')
    shutil.copytree(ROOT/'.tools/analyzers', tree/'.tools/analyzers')
    execute(side+'-independent', uv(str(TRIAL/'expected_runner.py')), tree, tree)
    execute(side+'-import', uv('-c', 'import lesscode_baseline; print(lesscode_baseline.__file__)'), tree, tree)
    execute(side+'-relevant', uv('-m', 'unittest', 'tests.test_contracts', 'tests.test_rule_contracts',
        'tests.test_rule_evidence', 'tests.test_rule_evaluation', '-q'), tree, tree)
    execute(side+'-regressions', uv('-m', 'unittest', 'discover', '-q'), tree, tree)


if __name__ == '__main__':
    command = sys.argv[1]
    if command == 'freeze':
        freeze()
    elif command == 'baseline':
        assert independent('baseline')['exit_code'] == 0
        red()
    elif command == 'candidate':
        candidate()
    elif command == 'regressions':
        with ThreadPoolExecutor(max_workers=2) as pool:
            list(pool.map(regressions, ('baseline', 'candidate')))
    elif command == 'repaired':
        regressions('candidate-v2', 'candidate')
    elif command == 'final':
        TRIAL = EXT/'reviewed-trial-v2'
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(regressions, 'admitted-'+side, side) for side in ('baseline', 'candidate')]
            for future in futures:
                future.result()
