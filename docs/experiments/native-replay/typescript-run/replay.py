"""One bounded real TypeScript trial; reads upstream, edits only fresh copies."""
import argparse
import copy
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
from datetime import datetime, timezone

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SOURCE = ROOT.parent / 'OpenUltraCode'
spec = importlib.util.spec_from_file_location('ats_common', HERE.parent / 'ats-run/replay.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
read, write, sha = common.read, common.write, common.sha
ORIGINAL = '''function slugify(value: string): string {
  let slug = ""
  let needsSeparator = false

  for (const character of value.trim().toLowerCase()) {
    const isLowerAscii = character >= "a" && character <= "z"
    const isDigit = character >= "0" && character <= "9"

    if (isLowerAscii || isDigit) {
      if (needsSeparator && slug.length > 0) {
        slug += "-"
      }

      slug += character
      needsSeparator = false
    } else {
      needsSeparator = slug.length > 0
    }
  }

  return slug.length === 0 ? "unknown" : slug
}
'''
REPLACEMENT = '''function slugify(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "unknown"
}
'''


def replace_helper(text):
    if text.count(ORIGINAL) != 1 or not text.endswith(ORIGINAL):
        raise ValueError('terminal slugify helper drifted; stop rather than widening edit')
    return text[:-len(ORIGINAL)] + REPLACEMENT


def command(args, cwd, log, timeout=120):
    proc = subprocess.run(list(map(str, args)), cwd=cwd, capture_output=True, text=True, encoding='utf-8', timeout=timeout)
    log.write_text(proc.stdout + proc.stderr, encoding='utf-8')
    return dict(command=list(map(str, args)), cwd=str(cwd), code=proc.returncode,
                log=log.name, log_sha256=sha(log))


def native(out, args):
    record = command([common.NATIVE, *args], ROOT, out)
    write(out.with_suffix('.execution.json'), record)
    if record['code'] not in (0, 1, 2):
        raise RuntimeError(out.read_text())
    return record


def files():
    selected = set()
    for folder in ('src', 'tests', 'scripts', '.opencode/plugins', '.opencode/lib', '.opencode/tools'):
        selected.update(p for p in (SOURCE / folder).rglob('*') if p.is_file())
    selected.update(SOURCE / name for name in ('package.json', 'pnpm-lock.yaml', 'tsconfig.json'))
    return sorted(selected)


def source_hashes():
    return {p.relative_to(SOURCE).as_posix(): sha(p) for p in files()}


def review(result, side, root):
    """Review only actual owned call sites; other project calls stay contextual."""
    relationships = {}
    for identity, finding in result['comparison']['accounting']['unresolved'].items():
        prefix = side + '/src/degradation.ts:'
        if not identity.startswith(prefix):
            continue
        key = identity.removeprefix(side + '/')
        offset = int(key.rsplit(':', 1)[1])
        excerpt = (root / 'src/degradation.ts').read_bytes()[offset:offset + 160].decode('utf-8')
        # No arbitrary function is certified by this review: retained calls are
        # Array.filter/map/join, Date/toISOString and String native methods.
        known = ('input.notices.filter', 'notices.map', 'new Date', 'value.trim',
                 'value.toLowerCase', 'Array', 'formatCapabilityList', 'createNotice', 'slugify')
        if not any(excerpt.startswith(value) for value in known):
            raise ValueError('unreviewed owned call ' + key + ': ' + excerpt)
        relationships[key] = dict(schema_version=1, owned_targets=[],
            external_contract='ECMAScript standard Array/String/Date methods; owned callbacks and helpers retained in the whole module',
            rationale='Reviewed native call location and complete module: ' + excerpt.splitlines()[0] + '. ' + finding['explanation'])
    return relationships


def run(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = source_hashes()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=SOURCE, text=True).strip()
    dirty = subprocess.check_output(['git', 'status', '--porcelain'], cwd=SOURCE, text=True)
    node = Path(shutil.which('node')).resolve()
    loader = SOURCE / 'node_modules/tsx/dist/loader.mjs'
    tsc = SOURCE / 'node_modules/typescript/bin/tsc'
    for path in (loader, tsc):
        if not path.is_file():
            raise ValueError('existing installed prerequisite missing: ' + str(path))
    baseline, candidate = output / 'baseline', output / 'candidate'
    for path in files():
        target = baseline / path.relative_to(SOURCE)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
    categories = {name: ('tests' if name.startswith('tests/') else 'research_support' if name.startswith('scripts/')
                        else 'production' if name.endswith('.ts') else 'configuration') for name in before}
    owned = {name: dict(kind='whole_file', path=name) for name in ('src/degradation.ts', 'src/types.ts')}
    manifest = dict(schema_version=1, trial_id='TS01-OpenUltraCode-degradation-6.5',
        hypothesis='Replace the custom ASCII slug loop with existing JavaScript native string operations; retain every degradation API and useful type.',
        entry_points={'degradation': owned['src/degradation.ts']}, owned=owned, wider_boundary=['.'], categories=categories,
        exclusions={}, scope_review=dict(schema_version=1, rationale='Whole degradation module and imported type module; all project source retained in wider accounting.', relationships={}),
        checks={}, target=None, capability_inventory=None)
    write(output / 'manifest-inventory-probe.json', manifest)
    native(output / 'inventory-prepare.log', ['prepare', '--manifest', output/'manifest-inventory-probe.json', '--baseline', baseline, '--out', output/'inventory-probe.json'])
    initial = read(output/'inventory-probe.json')
    package = read(SOURCE/'package.json')
    installed = {}
    for group in ('dependencies', 'devDependencies'):
        installed[group] = {}
        for name, declared in package.get(group, {}).items():
            artifact = SOURCE / 'node_modules' / name / 'package.json'
            installed[group][name] = dict(declared=declared, installed=read(artifact)['version'] if artifact.exists() else None,
                manifest_sha256=sha(artifact) if artifact.exists() else None)
    inventory = dict(source_identity=initial['baseline_identity'], source_revision=revision, dirty_state=dirty,
        manifest_references={name:before[name] for name in ('package.json','pnpm-lock.yaml','tsconfig.json')},
        runtime=dict(node=subprocess.check_output([node, '--version'], text=True).strip(), executable=str(node), sha256=sha(node)),
        runtime_capabilities=['ECMAScript String.toLowerCase/replace and RegExp; no production dependency additions'],
        installed=installed, roles='All declared dependencies are development/build/test tooling; no declared production libraries.',
        exclusions='node_modules/tool state excluded from source snapshots; installed package manifests hashed separately.', proposed_changes=[])
    write(output/'inventory.json', inventory)
    type_args = [node, tsc, '--noEmit', '--typeRoots', SOURCE/'node_modules/@types']
    test_args = [node, '--import', loader.as_uri(), '--test', 'tests/degradation.test.ts']
    observe_args = [node, '--import', loader.as_uri(), HERE/'observe.mjs']
    definition = dict(runner_sha256=sha(Path(__file__)), observation_sha256=sha(HERE/'observe.mjs'),
        plan_sha256=sha(HERE/'plan.md'), test_sha256=before['tests/degradation.test.ts'],
        tsconfig_sha256=before['tsconfig.json'], type_args=list(map(str,type_args)), test_args=list(map(str,test_args)),
        observe_args=list(map(str,observe_args)), loader_sha256=sha(loader), compiler_sha256=sha(tsc), node=inventory['runtime'])
    write(output/'check-definition.json', definition)
    for name in ('types','project-tests','differential'):
        manifest['checks'][name] = dict(schema_version=1, description=name+' frozen unchanged contract check', required=True,
            definition_digest=sha(output/'check-definition.json'), artifact_expectations=[], interpretation=dict(kind='exit_status'))
    defs = {v['name']:v for v in initial['definitions'].values() if v['language']=='type_script' and v['name']!='internal_call_layers'}
    manifest['target'] = dict(schema_version=1, scope_identity='production', language='type_script', definitions=defs,
        minimum_production_statement_reduction=1, count_bounds={}, permitted_structural_tradeoffs=['Whole modules and wider source count; unknown call layers contextual. No TS project-wide reduction or maintenance claim.'])
    manifest['capability_inventory'] = dict(schema_version=1, source_identity=initial['baseline_identity'],
        manifest_references=['package.json','pnpm-lock.yaml','tsconfig.json'],
        artifact=dict(schema_version=1,path='inventory.json',sha256=sha(output/'inventory.json')))
    # Freeze target/inventory/checks before constructing or observing the candidate.
    write(output/'manifest-before-edit.json', manifest)
    native(output/'prepare-before-edit.log', ['prepare','--manifest',output/'manifest-before-edit.json','--baseline',baseline,'--artifact-root',output,'--out',output/'frozen-before-edit.json'])
    write(output/'edit-start.json',dict(utc=datetime.now(timezone.utc).isoformat(), budget_seconds=7200))
    shutil.copytree(baseline, candidate)
    p = candidate/'src/degradation.ts'
    old = p.read_text(encoding='utf-8')
    p.write_text(replace_helper(old), encoding='utf-8', newline='\n')
    (output/'candidate.patch').write_text(''.join(difflib.unified_diff(old.splitlines(True),p.read_text().splitlines(True),fromfile='baseline/src/degradation.ts',tofile='candidate/src/degradation.ts')),encoding='utf-8')
    raw = {}
    for arm, work in (('baseline',baseline),('candidate',candidate)):
        folder = output/(arm+'-raw'); folder.mkdir()
        raw[arm] = {}
        for name,args in (('types',type_args),('project-tests',test_args),('observations',[*observe_args,work,folder/'observations.json'])):
            raw[arm][name] = command(args,work,folder/(name+'.log'))
            write(folder/(name+('-execution' if name=='observations' else '')+'.json'),raw[arm][name])
    write(output/'paired-check-receipt.json', dict(source_before_sha256=before,source_after_sha256=source_hashes(),
        source_revision=revision,dirty_state=dirty,check_definition_sha256=sha(output/'check-definition.json'),checks=raw))
    evidence = dict(schema_version=1, candidate_ownership=dict(schema_version=1,replacements={name:[value] for name,value in owned.items()},review=copy.deepcopy(manifest['scope_review'])), checks={}, dependency_changes={})
    write(output/'empty-evidence.json', evidence)
    native(output/'probe-compare.log',['compare','--frozen',output/'frozen-before-edit.json','--baseline',baseline,'--candidate',candidate,'--evidence',output/'empty-evidence.json','--artifact-root',output,'--out',output/'scope-probe'])
    probe = read(output/'scope-probe/result.json')
    manifest['scope_review']['relationships'] = review(probe,'baseline',baseline)
    evidence['candidate_ownership']['review']['relationships'] = review(probe,'candidate',candidate)
    # Only relationship annotations change after source inspection. Targets,
    # source identities and check definitions remain byte-for-byte equivalent.
    write(output/'manifest.json',manifest)
    native(output/'prepare.log',['prepare','--manifest',output/'manifest.json','--baseline',baseline,'--artifact-root',output,'--out',output/'frozen.json'])
    frozen = read(output/'frozen.json')
    trial = hashlib.sha256(json.dumps(frozen,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
    identities = {'baseline': frozen['baseline_identity']}
    no_inventory = copy.deepcopy(manifest)
    no_inventory['capability_inventory'] = None
    write(output/'identity-manifest.json',no_inventory)
    native(output/'candidate-identity.log',['prepare','--manifest',output/'identity-manifest.json','--baseline',candidate,'--out',output/'candidate-identity.json'])
    identities['candidate'] = read(output/'candidate-identity.json')['baseline_identity']
    baseline_obs = read(output/'baseline-raw/observations.json')
    for arm in ('baseline','candidate'):
        obs = read(output/(arm+'-raw/observations.json'))
        mismatches = sum(a!=b for a,b in zip(baseline_obs['digests'],obs['digests'])) + abs(baseline_obs['cases']-obs['cases'])
        summary = dict(code=0 if mismatches==0 else 1,cases=obs['cases'],mismatches=mismatches)
        raw[arm]['differential'] = summary
        write(output/(arm+'-raw/differential.json'),summary)
        for name in manifest['checks']:
            pair = evidence['checks'].setdefault(name,dict(schema_version=1,baseline=None,candidate=None))
            result = raw[arm][name]
            paths = [output/(arm+'-raw')/(name+'.json'),output/(arm+'-raw/observations.json' if name=='differential' else arm+'-raw/'+name+'.log')]
            pair[arm] = dict(schema_version=1, source_identity=identities[arm], frozen_trial_digest=trial,
                definition_digest=sha(output/'check-definition.json'), status='passed' if result['code']==0 else 'failed',
                summary=dict(kind='exit_status',code=result['code']), artifacts={p.name:dict(schema_version=1,path=p.relative_to(output).as_posix(),sha256=sha(p)) for p in paths},
                tested_domain='Existing degradation Node tests, strict existing project build and '+str(obs['cases'])+' finite public API observations.',
                limitations=['No external host/permission execution; no universal equivalence.', 'Installed dependency source is external, separately inventoried; no dependencies added.'])
    write(output/'candidate-evidence.json',evidence)
    native(output/'compare.log',['compare','--frozen',output/'frozen.json','--baseline',baseline,'--candidate',candidate,'--evidence',output/'candidate-evidence.json','--artifact-root',output,'--out',output/'comparison'])
    mutant = output/'mutant'; shutil.copytree(candidate,mutant)
    p = mutant/'src/degradation.ts'; p.write_text(p.read_text().replace('/[^a-z0-9]+/g','/[^a-z0-9]/g'),encoding='utf-8')
    folder=output/'mutant-raw'; folder.mkdir()
    mutant_command = command([*observe_args,mutant,folder/'observations.json'],mutant,folder/'observations.log')
    obs = read(folder/'observations.json')
    mismatches = sum(a!=b for a,b in zip(baseline_obs['digests'],obs['digests']))
    write(folder/'negative-control.json',dict(command=mutant_command,cases=obs['cases'],mismatches=mismatches,detected=mismatches>0))
    if mismatches==0: raise ValueError('frozen corpus failed to detect separator mutant')
    after = source_hashes()
    if before!=after or dirty!=subprocess.check_output(['git','status','--porcelain'],cwd=SOURCE,text=True):
        raise ValueError('upstream source or dirty state changed')
    if definition['runner_sha256']!=sha(Path(__file__)) or definition['observation_sha256']!=sha(HERE/'observe.mjs'):
        raise ValueError('checks changed during run')
    write(output/'receipt.json',dict(finished_utc=datetime.now(timezone.utc).isoformat(),source_root=str(SOURCE),
        source_revision=revision,dirty_state=dirty,source_before_sha256=before,source_after_sha256=after,identities=identities,
        native_executable_sha256=sha(common.NATIVE),runner_sha256=sha(Path(__file__)),observations=baseline_obs['cases'],
        mutant_mismatches=mismatches,status=read(output/'comparison/result.json')['comparison']['status'],adopted=False))
    return read(output/'receipt.json')['status']


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out',type=Path,required=True)
    print(run(parser.parse_args().out))
