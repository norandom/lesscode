"""Located call review for the two complete owned contracts modules.

Classification rules are explicit for this responsibility, not a general resolver.
Every record retains the expression, source line and declared binding. Dynamic
callbacks remain dynamic; no call-layer number is inferred from these reviews.
"""
import ast
import builtins
import json
from pathlib import Path
import shutil

from conduct import ROOT, RUN, EXT, HERE, TRIAL, dump, execute, sha

OWNED = ['src/lesscode_baseline/contracts.py', 'src/lesscode_baseline/rules/contracts.py']
CALLBACKS = {'reader', 'identity', 'cls', 'getattr', 'type(record).from_dict',
             'record_type.from_dict', 'readers[kind].from_dict', 'readers[operation].from_dict',
             "{'numeric': NumericRule, 'check': CheckRule, 'review': ReviewRule}[kind].from_dict"}
STANDARD = {
    'datetime': 'datetime.datetime', 'timedelta': 'datetime.timedelta',
    'PurePosixPath': 'pathlib.PurePosixPath', 'PureWindowsPath': 'pathlib.PureWindowsPath',
    'asdict': 'dataclasses.asdict', 'dataclass': 'dataclasses.dataclass',
    'json': 'json', 'math': 'math', 're': 're', 'hashlib': 'hashlib', 'base64': 'base64',
}
CONTAINERS = {'value', 'fields', 'required', 'optional', 'names', 'seen', 'roles', 'rules',
              'counts', 'd', 'addresses', 'address', 'filename', 'origin', 'p', 'part',
              'resource', 'x', 'text', 'parsed', 'windows', 'instant'}


def review(side):
    output = {}
    expressions = {}
    for relative in OWNED:
        source = (RUN/side/relative).read_text()
        tree = ast.parse(source)
        definitions = {node.name for node in tree.body if isinstance(node, (ast.ClassDef, ast.FunctionDef))}
        imports = {}
        for node in tree.body:
            if isinstance(node, ast.ImportFrom):
                for name in node.names:
                    imports[name.asname or name.name] = (node.module, name.name)
        offsets = [0]
        for line in source.splitlines(keepends=True):
            offsets.append(offsets[-1] + len(line.encode('utf-8')))
        # Native offsets refer to bytes on disk, including CRLF in baseline.
        raw = (RUN/side/relative).read_bytes()
        raw_offsets = [0]
        for line in raw.splitlines(keepends=True):
            raw_offsets.append(raw_offsets[-1] + len(line))
        for call in (n for n in ast.walk(tree) if isinstance(n, ast.Call)):
            expression = ast.unparse(call.func)
            root = expression.split('.')[0].split('(')[0]
            binding = None
            targets = []
            external = None
            if expression in CALLBACKS:
                targets = OWNED.copy()
                external = 'Caller-supplied callable/record subclass protocol: ' + expression
                binding = 'Dynamic callback/constructor dispatch; selected records and validators owned, externally supplied callbacks remain caller responsibility. Invocation order and subclass dispatch retained.'
            elif root in definitions:
                targets = [relative]
                binding = 'Locally defined selected-module implementation: ' + root
            elif root in imports and imports[root][0] == 'lesscode_baseline.contracts':
                targets = [OWNED[0]]
                binding = 'Imported selected baseline contract implementation: ' + imports[root][1]
            elif root == 'matches':
                external = 'Unchanged lesscode_baseline.snapshot.matches POSIX glob contract'
                binding = 'Concrete retained snapshot dependency; unchanged source remains inside wider production accounting, not a replacement helper.'
            elif root in STANDARD:
                external = STANDARD[root] + ': ' + expression
                binding = 'Existing Python standard-library capability; no new dependency or implementation relocation.'
            elif root in vars(builtins):
                external = 'Python builtin ' + expression
                binding = 'Existing builtin/container protocol; custom object callback behavior outside finite checks remains unproved.'
            elif root in CONTAINERS or isinstance(call.func, ast.Attribute) and isinstance(call.func.value, (ast.Constant, ast.Set)):
                external = 'Python value/container/string/datetime protocol: ' + expression
                binding = 'Operation on validated fields or local containers; arbitrary custom subclasses remain an explicit finite-domain limitation.'
            elif expression in ('evidence.to_dict', 'policy.to_dict', 'r.to_dict'):
                targets = OWNED.copy()
                external = 'Record.to_dict virtual dispatch on caller-supplied subclass'
                binding = 'Selected Record serialization and typed records owned; virtual overrides retained and not statically resolved.'
            elif expression.startswith(('PurePosixPath(', 'PureWindowsPath(', 'base64.', 'hashlib.', 'dumps(')):
                external = 'Python standard-library result protocol: ' + expression
                binding = 'Result operation of existing pathlib/base64/hashlib/JSON serialization; dumps implementation owned in selected baseline module.'
                if expression.startswith('dumps('):
                    targets = [OWNED[0]]
            else:
                raise RuntimeError('Unreviewed expression: '+relative+':'+str(call.lineno)+' '+expression)
            key = relative+':'+str(raw_offsets[call.lineno-1] + call.col_offset)
            output[key] = dict(schema_version=1, owned_targets=targets, external_contract=external,
                rationale=f'{side} line {call.lineno}: {expression}. {binding}')
            expressions.setdefault(expression, dict(owned_targets=targets, external_contract=external))
        for function in (n for n in ast.walk(tree) if isinstance(n, (ast.FunctionDef, ast.ClassDef))):
            for decorator in function.decorator_list:
                if isinstance(decorator, ast.Name):
                    assert decorator.id in ('classmethod', 'staticmethod', 'property')
                    key = relative+':'+str(raw_offsets[decorator.lineno-1] + decorator.col_offset - 1)
                    output[key] = dict(schema_version=1, owned_targets=[],
                        external_contract='Python builtin descriptor '+decorator.id,
                        rationale=f'{side} line {decorator.lineno}: @{decorator.id} binds selected method; implicit decorator invocation, not an omitted helper.')
    return output, expressions


if __name__ == '__main__':
    baseline, baseline_expressions = review('baseline')
    candidate, candidate_expressions = review('candidate')
    reviewed = EXT/'reviewed-trial-v2'
    reviewed.mkdir(exist_ok=False)
    for name in ('inventory.json', 'check-definition.json', 'expected_runner.py', 'expected-outcomes.json'):
        shutil.copyfile(TRIAL/name, reviewed/name)
    manifest = json.loads((TRIAL/'manifest.json').read_text())
    manifest['trial_id'] += '-scope-reviewed'
    manifest['scope_review']['relationships'] = baseline
    manifest['scope_review']['rationale'] += ' Located call review finalized after first candidate edit; unchanged checks/evaluator/target were frozen before editing. See scope-review-provenance.json.'
    dump(reviewed/'manifest.json', manifest)
    dump(HERE/'scope-review-expressions.json', dict(baseline=baseline_expressions, candidate=candidate_expressions))
    dump(HERE/'candidate-scope-review.json', dict(schema_version=1,
        rationale='Both entire contracts modules, wrappers and shared implementations owned. Explicit located call reviews; dynamic callback contracts retained; no new helpers or changed callers.', relationships=candidate))
    execute('reviewed-prepare', [str(EXT/'lesscode-core.exe'), 'prepare', '--manifest', str(reviewed/'manifest.json'),
        '--baseline', str(RUN/'baseline'), '--artifact-root', str(reviewed), '--out', str(reviewed/'frozen.json')], ROOT)
    for name in ('manifest.json', 'frozen.json'):
        shutil.copyfile(reviewed/name, HERE/('reviewed-'+name))
    dump(HERE/'scope-review-provenance.json', dict(
        initial_pre_edit_frozen_sha256=sha(TRIAL/'frozen.json'),
        revised_frozen_sha256=sha(reviewed/'frozen.json'),
        revised_check_definition_sha256=sha(reviewed/'check-definition.json'),
        pre_edit_check_definition_sha256=sha(TRIAL/'check-definition.json'),
        checks_bytes_unchanged=sha(reviewed/'check-definition.json')==sha(TRIAL/'check-definition.json'),
        chronology='Scope review completed post candidate; final paired checks start only after this reviewed freeze. Preliminary runs are not reused.',
        baseline_call_records=len(baseline), candidate_call_records=len(candidate)))
