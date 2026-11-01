"""Bounded fresh NPU software-only replay; historical collectors are never run."""
import argparse
import ast
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
spec = importlib.util.spec_from_file_location('ats_adapter', HERE.parent / 'ats-run/replay.py')
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
read, write, sha, cli = common.read, common.write, common.sha, common.cli
INPUTS = ROOT / '.runs/native-replay/task-6.1/npu'
ARCHIVE = ROOT / 'docs/experiments/npu-backends'
PYTHON = ROOT.parent / 'npu_rag_from_folder/.venv/Scripts/python.exe'
PREFIX = 'src/npu_rag/embedding/providers/'
TESTS = ['tests/embedding/providers/test_cpu.py', 'tests/embedding/providers/test_vitisai.py', 'tests/embedding/providers/test_base.py', 'tests/embedding/test_service.py', 'tests/embedding/test_postprocess.py']


def contract_for(expression):
    if expression in {'isinstance', 'len', 'list', 'str', 'tuple', 'type', 'sum', 'FileNotFoundError', 'ValueError', 'property', 'staticmethod'}:
        return 'Python builtin ' + expression
    contracts = {
        "', '.join": 'Python str.join', 'Path': 'pathlib.Path construction',
        'Path(ort.__file__).parent.joinpath': 'pathlib.Path.joinpath', 'context.is_file': 'pathlib.Path.is_file',
        'resolved.is_file': 'pathlib.Path.is_file', 'context_ops.get': 'Python dict.get',
        'context_ops.items': 'Python dict.items', 'counts.get': 'Python dict.get',
        'ort.InferenceSession': 'onnxruntime.InferenceSession', 'ort.get_available_providers': 'onnxruntime.get_available_providers',
        'probe_onnx_runtime': 'Unchanged npu_rag.embedding.environment.capability.probe_onnx_runtime software availability API', 'onnx.load': 'onnx.load', 'session.get_providers': 'onnxruntime.InferenceSession.get_providers',
        'self._session.run': 'onnxruntime.InferenceSession.run', 'session_run': 'Callable supplied by provider: onnxruntime.InferenceSession.run',
        'build': 'Injected SessionFactory callable; defaults to provider _open',
        'available_providers': 'Injected provider enumeration callable; defaults to onnxruntime.get_available_providers',
        'probe': 'Injected partition probe callable; defaults to owned probe_onnx_runtime',
        'read': 'Injected graph reader callable; defaults to owned _load_graph',
    }
    if expression in contracts:
        return contracts[expression]
    raise ValueError('unreviewed call contract: ' + expression)


def relationships(root, ids):
    modules = {p: ast.parse((root / p).read_text(encoding='utf-8')) for p in (PREFIX + 'cpu.py', PREFIX + 'vitisai.py', PREFIX + 'checked_run.py') if (root / p).exists()}
    result = {}
    for identity in ids:
        path, offset = identity.rsplit(':', 1)
        if path not in modules:
            continue # Wider unchanged project relationships remain contextual and unreviewed.
        raw = (root / path).read_bytes()
        starts = [0]
        for line in raw.splitlines(keepends=True):
            starts.append(starts[-1] + len(line))
        nodes = [n for n in ast.walk(modules[path]) if isinstance(n, ast.Call)]
        nodes += [n for fn in ast.walk(modules[path]) if isinstance(fn, (ast.FunctionDef, ast.ClassDef)) for n in fn.decorator_list if not isinstance(n, ast.Call)]
        call = next((n for n in nodes if starts[n.lineno-1] + n.col_offset == int(offset) or starts[n.lineno-1] + n.col_offset - 1 == int(offset)), None)
        if call is None:
            raise ValueError('unreviewed native location: ' + identity)
        expression = ast.unparse(call.func if isinstance(call, ast.Call) else call)
        name = expression.removeprefix('self.')
        owned = [p for p, tree in modules.items() if any(isinstance(n, ast.FunctionDef) and n.name == name for n in ast.walk(tree))]
        imported = {'EnvironmentError_': 'src/npu_rag/embedding/errors.py', 'ExecutionError': 'src/npu_rag/embedding/errors.py',
                    'PartitionShareTooLow': 'src/npu_rag/embedding/errors.py', 'ProviderChoice': 'src/npu_rag/embedding/types.py'}
        if expression in imported:
            # These constructors are unchanged owned dependencies in the wider root.
            contract = 'Unchanged owned API ' + imported[expression] + ':' + expression
        elif expression == 'failure':
            contract = 'Callable bound to provider._failed, counted in the owned provider modules'
        else:
            contract = None if owned else contract_for(expression)
        result[identity] = dict(schema_version=1, owned_targets=sorted({PREFIX+'cpu.py' if p == PREFIX+'checked_run.py' else p for p in owned}), external_contract=contract,
            rationale=f'Reviewed {expression} at {path}:{offset}; ' + ('whole owned implementation counted.' if owned else contract + '; constructor/interaction software checks only, no hardware claim.'))
    return result


def observe(work, out):
    sys.path.insert(0, str(work / 'src'))
    os.chdir(work)
    # Exact archived generated corpus statements; replace only historical path/output plumbing.
    text = (ARCHIVE / 'run.py').read_text(encoding='utf-8')
    body = text[text.index('import numpy as np'):text.index('(HERE / arm / "outcomes.json")')]
    namespace = {'itertools': __import__('itertools')}
    exec(compile(body, 'frozen-npu-corpus', 'exec'), namespace)
    actual = namespace['outcomes']
    expected = read(ARCHIVE / 'original/outcomes.json')
    mismatches = [i for i, (old, new) in enumerate(zip(expected, actual)) if old != new]
    write(out, dict(cases=len(actual), expected=len(expected), mismatches=len(mismatches),
        outcome_digests=[hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest() for value in actual], witnesses=[dict(case=i, expected=expected[i], actual=actual[i]) for i in mismatches[:4]]))


def command(work, args, python, logfile):
    env = dict(os.environ, PYTHONPATH=str(work / 'src'), MYPYPATH=str(work / 'src'), PYTEST_DISABLE_PLUGIN_AUTOLOAD='1', PYTHONDONTWRITEBYTECODE='1', MYPY_CACHE_DIR=str(logfile.parent / 'mypy-cache'))
    proc = subprocess.run(['uv.exe', 'run', '--no-project', '--no-python-downloads', '--python', str(python.resolve()), 'python', *map(str, args)],
                          cwd=work, env=env, capture_output=True, text=True, timeout=180)
    logfile.write_text(proc.stdout + proc.stderr, encoding='utf-8')
    return proc.returncode


def safety(baseline, candidate):
    for name in ('cpu', 'vitisai'):
        def retained(root):
            tree = ast.parse((root / (PREFIX + name + '.py')).read_text(encoding='utf-8'))
            cls = next(n for n in tree.body if isinstance(n, ast.ClassDef))
            return [ast.dump(n) for n in cls.body if getattr(n, 'name', None) not in {'run', '_check', '_only_tensor'}]
        if retained(baseline) != retained(candidate):
            return dict(code=1, detail='retained provider construction/API changed')
    helper = candidate / (PREFIX + 'checked_run.py')
    if helper.exists():
        calls = [ast.unparse(n.func) for n in ast.walk(ast.parse(helper.read_text())) if isinstance(n, ast.Call)]
        forbidden = [n for n in calls if n.endswith(('astype', 'SessionOptions', 'add_session_config_entry')) or any(p in n for p in ('quantiz', 'np.mean', 'np.sum', 'np.linalg', 'print', 'logging'))]
        if forbidden:
            return dict(code=1, detail=forbidden)
    return dict(code=0, detail='unchanged retained provider members; no forbidden shared-helper calls')


def bound(rawpath, source, trial, definition, check, output):
    raw = read(rawpath)
    if check == 'differential':
        passed = raw['cases'] == 3168 and raw['mismatches'] == 0
        summary = dict(kind='case_counts', expected=3168, observed=raw['cases'], mismatches=raw['mismatches'])
    else:
        passed = raw['code'] == 0
        summary = dict(kind='exit_status', code=raw['code'])
    artifacts = {'raw': dict(schema_version=1, path=rawpath.relative_to(output).as_posix(), sha256=sha(rawpath))}
    for name in raw.get('artifacts', []):
        p = rawpath.parent / name
        artifacts[name] = dict(schema_version=1, path=p.relative_to(output).as_posix(), sha256=sha(p))
    return dict(schema_version=1, source_identity=source, frozen_trial_digest=trial, definition_digest=definition,
        status='passed' if passed else 'failed', summary=summary, artifacts=artifacts,
        tested_domain='3168 frozen tensor boundary cases; 339 unchanged software provider/service/postprocess tests; selected mypy modules; retained-member/forbidden-call safety audit',
        limitations=['Finite software-only evidence; no hardware or performance claim.', 'Hashes bind supplied artifacts; they do not authenticate execution.'])


def run(output, inputs, python):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    before = common.prepare.original_hashes()
    input_hashes = {p.relative_to(inputs).as_posix(): sha(p) for p in inputs.rglob('*') if p.is_file()}
    for arm in ('baseline', 'candidate', 'mutant'):
        shutil.copytree(inputs / arm, output / arm)
    definition = dict(schema_version=1, runner_sha256=sha(Path(__file__)), archived_runner_sha256=sha(ARCHIVE / 'run.py'),
        oracle_sha256=sha(ARCHIVE / 'original/outcomes.json'), python=str(python.resolve()), tests={p: sha(output / 'baseline' / p) for p in TESTS},
        test_args=['-m', 'pytest', '-q', '--import-mode=importlib', '-p', 'no:cacheprovider', *TESTS],
        type_args=['-m', 'mypy', '--follow-imports=silent', '--no-incremental', PREFIX+'cpu.py', PREFIX+'vitisai.py'],
        candidate_type_addition=PREFIX+'checked_run.py', safety='unchanged retained members and archived forbidden-call policy')
    write(output / 'check-definition.json', definition)
    manifest = read(inputs / 'manifest.json')
    manifest['trial_id'] = 'NPU01-fresh-native-source-target-6.3'
    del manifest['target']['definitions']['internal_call_layers']
    manifest['target']['permitted_structural_tradeoffs'].append('Separately authorized 6.3 source-metric target. Unknown internal call layers remain contextual and unavailable; stricter 6.1 all-metric target remains incomplete.')
    for check in manifest['checks'].values():
        check['definition_digest'] = sha(output / 'check-definition.json')
        check['artifact_expectations'] = []
    write(output / 'manifest-probe.json', manifest)
    cli(['prepare', '--manifest', output/'manifest-probe.json', '--baseline', output/'baseline', '--out', output/'probe-frozen.json'])
    for arm in ('candidate', 'mutant'):
        cli(['compare', '--frozen', output/'probe-frozen.json', '--baseline', output/'baseline', '--candidate', output/arm, '--evidence', inputs/'candidate-evidence.json', '--out', output/(arm+'-probe')], (0,1,2,3))
    def reviewed(arm, side):
        probe = read(output/(arm+'-probe/result.json'))['comparison']
        return relationships(output/('baseline' if side=='baseline' else arm), [key.removeprefix(side+'/') for key in probe['accounting']['unresolved'] if key.startswith(side+'/')])
    manifest['scope_review']['relationships'] = reviewed('candidate', 'baseline')
    write(output/'manifest.json', manifest)
    cli(['prepare', '--manifest', output/'manifest.json', '--baseline', output/'baseline', '--out', output/'frozen.json'])
    frozen = read(output/'frozen.json')
    trial = hashlib.sha256(json.dumps(frozen, ensure_ascii=False, separators=(',', ':')).encode()).hexdigest()
    identities = {}
    # The final target and required inventory are frozen before any fresh execution.
    for arm in ('baseline', 'candidate', 'mutant'):
        cli(['prepare', '--manifest', output/'manifest.json', '--baseline', output/arm, '--out', output/(arm+'-identity.json')])
        identities[arm] = read(output/(arm+'-identity.json'))['baseline_identity']
        rawdir = output/(arm+'-raw')
        rawdir.mkdir()
        code = command(output/arm, [str(Path(__file__)), 'observe', '--work', str(output/arm), '--out', str(rawdir/'differential.json')], python, rawdir/'corpus.log')
        if code:
            raise RuntimeError((rawdir/'corpus.log').read_text())
        if arm != 'mutant':
            code = command(output/arm, [*definition['test_args'][:6], '--junitxml='+str(rawdir/'tests.xml'), *TESTS], python, rawdir/'tests.log')
            suite = ET.parse(rawdir/'tests.xml').getroot().find('testsuite')
            write(rawdir/'native-tests.json', dict(code=code, tests=int(suite.attrib['tests']), failures=int(suite.attrib['failures']), artifacts=['tests.log', 'tests.xml']))
            args = definition['type_args'] + ([definition['candidate_type_addition']] if arm=='candidate' else [])
            code = command(output/arm, args, python, rawdir/'types.log')
            write(rawdir/'types.json', dict(code=code, artifacts=['types.log']))
            write(rawdir/'shared-safety.json', safety(output/'baseline', output/arm))
    outcomes = {}
    template = read(inputs/'candidate-evidence.json')
    for arm in ('candidate','mutant'):
        evidence = copy.deepcopy(template)
        evidence['candidate_ownership']['review']['relationships'] = reviewed(arm, 'candidate')
        for check, declared in manifest['checks'].items():
            for side, actual in (('baseline','baseline'), ('candidate',arm)):
                if actual == 'mutant' and check != 'differential':
                    continue # Not run: known behavior rejection takes precedence over missing evidence.
                evidence['checks'][check][side] = bound(output/(actual+'-raw')/(check+'.json'), identities[actual], trial, declared['definition_digest'], check, output)
        write(output/(arm+'-evidence.json'), evidence)
        cli(['compare','--frozen',output/'frozen.json','--baseline',output/'baseline','--candidate',output/arm,'--evidence',output/(arm+'-evidence.json'),'--artifact-root',output,'--out',output/(arm+'-comparison')], (0,1,2,3))
        outcomes[arm] = read(output/(arm+'-comparison/result.json'))['comparison']['status']
    for name in ('moved-helper','scope-narrowing','classification','missing-check','stale-evidence'):
        candidate = output/'candidate'
        evidence = copy.deepcopy(read(output/'candidate-evidence.json'))
        if name in ('moved-helper','classification'):
            candidate = output/name
            shutil.copytree(output/'candidate',candidate)
            original = candidate/(PREFIX+'checked_run.py')
            target = candidate/('support/moved_helper.py' if name=='moved-helper' else 'tests/hidden_implementation.py')
            target.parent.mkdir(parents=True,exist_ok=True)
            original.replace(target)
            # Actual relocation out of selected files; no added category admits the hidden implementation.
        elif name=='scope-narrowing':
            evidence['candidate_ownership']['replacements'].clear()
        elif name=='missing-check':
            evidence['checks']['types']['candidate'] = None
        else:
            evidence['checks']['differential']['candidate']['source_identity'] = identities['baseline']
        write(output/(name+'-evidence.json'),evidence)
        args = ['compare','--frozen',output/'frozen.json','--baseline',output/'baseline','--candidate',candidate,'--evidence',output/(name+'-evidence.json'),'--artifact-root',output,'--out',output/(name+'-comparison')]
        execution = subprocess.run([str(common.NATIVE), *map(str,args)], capture_output=True, text=True, timeout=120)
        write(output/(name+'-native-execution.json'), dict(command=list(map(str,args)), code=execution.returncode, stdout=execution.stdout, stderr=execution.stderr))
        result_path = output/(name+'-comparison/result.json')
        outcomes[name] = read(result_path)['comparison']['status'] if result_path.exists() else 'invalid'
        if not result_path.exists() and execution.returncode != 3:
            raise RuntimeError(execution.stderr)
    after = common.prepare.original_hashes()
    if before != after:
        raise ValueError('historical originals changed')
    if input_hashes != {p.relative_to(inputs).as_posix(): sha(p) for p in inputs.rglob('*') if p.is_file()}:
        raise ValueError('prepared inputs changed')
    write(output/'outcomes.json',outcomes)
    write(output/'receipt.json',dict(originals_before_sha256=before, originals_after_sha256=after, source_identities=identities,
        prepared_inputs_before_sha256=input_hashes, prepared_inputs_after_sha256=input_hashes, runner_sha256=sha(Path(__file__)), native_executable_sha256=sha(common.NATIVE), python=str(python.resolve())))
    return outcomes

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    replay = sub.add_parser('run')
    replay.add_argument('--out', type=Path, required=True)
    replay.add_argument('--inputs', type=Path, default=INPUTS)
    replay.add_argument('--python', type=Path, default=PYTHON)
    observation = sub.add_parser('observe')
    observation.add_argument('--work', type=Path, required=True)
    observation.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'observe':
        observe(args.work,args.out)
    else:
        run(args.out,args.inputs,args.python)






