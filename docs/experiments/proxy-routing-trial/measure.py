"""Count the full changed module and frozen project's declared production files."""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

record = Path(__file__).parent
run = Path('.runs/proxy-routing-trial')
snapshot = json.loads((record / 'snapshot.json').read_text(encoding="utf-8"))
roots = ('mvibe_auth/', 'mvibe_proxy/', 'packages/opencode-vibe-oidc/src/')

def counts(text):
    tree = ast.parse(text)
    return {'physical_lines': len(text.splitlines()),
            'ast_statements': sum(isinstance(node, ast.stmt) for node in ast.walk(tree)),
            'decisions': sum(isinstance(node, (ast.If, ast.IfExp, ast.For, ast.AsyncFor,
                ast.While, ast.ExceptHandler, ast.comprehension)) +
                (len(node.values) - 1 if isinstance(node, ast.BoolOp) else 0)
                for node in ast.walk(tree)),
            'functions': sum(isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) for node in ast.walk(tree))}

result = {}
texts = {}
for side in ('before', 'after'):
    root = run / side
    text = (root / 'mvibe_auth/http.py').read_text(encoding="utf-8")
    texts[side] = text
    tree = ast.parse(text)
    selected = [node for node in tree.body if getattr(node, 'name', '').startswith(('_EnvProxy', 'VibeAsync', '_normalize_proxy', '_should_bypass', '_no_proxy', '_ip_', '_host_rule', '_parse_no_proxy'))]
    responsibility = '\n'.join('\n'.join(text.splitlines()[node.lineno-1:node.end_lineno]) for node in selected)
    production = [name for name in snapshot['sha256'] if (name.startswith(roots) or name == 'proxy.py')
                  and Path(name).suffix in ('.py', '.ts', '.tsx')]
    tests = [name for name in snapshot['sha256'] if name.startswith('tests/')]
    result[side] = {'module': counts(text), 'responsibility': counts(responsibility),
                    'project_production_physical_lines': sum(len((root/name).read_text(encoding='utf-8-sig').splitlines()) for name in production),
                    'production_files': len(production),
                    'unchanged_test_files': len(tests),
                    'changed_snapshot_files': [name for name, digest in snapshot['sha256'].items()
                        if hashlib.sha256((root/name).read_bytes()).hexdigest() != digest]}
    assert all(hashlib.sha256((root/name).read_bytes()).hexdigest() == snapshot['sha256'][name] for name in tests)
for side, label in [('before', 'baseline'), ('after', 'candidate')]:
    report = ET.parse(run / f'{label}-tests.xml').getroot()
    result[side]['tests'] = [element.attrib for element in report.findall('testsuite')]
    result[side]['test_exit'] = int((run / f'{label}-tests.exit').read_text(encoding="utf-8"))
(record / 'candidate.patch').write_text(''.join(difflib.unified_diff(texts['before'].splitlines(True), texts['after'].splitlines(True), fromfile='a/mvibe_auth/http.py', tofile='b/mvibe_auth/http.py')))
(record / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
