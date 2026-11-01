"""Retain whole-scope counts, paired test dispositions and source identities."""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

record = Path(__file__).parent
run = Path('.runs/facdrone-csv-trial')
snapshot = json.loads((record / 'snapshot.json').read_text(encoding="utf-8"))
relative = 'packages/facdrone-reporting/facdrone_reporting/library/normalized_csv.py'
result, texts = {}, {}
for side in ('before', 'after'):
    root = run / side
    text = (root / relative).read_text(encoding="utf-8")
    texts[side] = text
    tree = ast.parse(text)
    package = [name for name in snapshot['sha256']
               if name.startswith('packages/facdrone-reporting/facdrone_reporting/')
               and '/testing/' not in name and name.endswith('.py')]
    cases = ET.parse(run / f'{side}.xml').getroot().findall('.//testcase')
    tests = [{'name': item.attrib['classname'] + '.' + item.attrib['name'],
              'status': 'failed' if item.find('failure') is not None else 'passed',
              'error': item.find('failure').attrib.get('message', '') if item.find('failure') is not None else None}
             for item in cases]
    result[side] = {
        'module': {'physical_lines': len(text.splitlines()),
                   'ast_statements': sum(isinstance(node, ast.stmt) for node in ast.walk(tree)),
                   'ast_nodes': sum(1 for _ in ast.walk(tree)),
                   'functions': sum(isinstance(node, ast.FunctionDef) for node in ast.walk(tree))},
        'package_production_python_files': len(package),
        'package_production_python_lines': sum(len((root/name).read_text(encoding='utf-8-sig').splitlines()) for name in package),
        'changed_tracked_files': [name for name, digest in snapshot['sha256'].items()
            if hashlib.sha256((root/name).read_bytes()).hexdigest() != digest],
        'tests': tests, 'test_exit': int((run/f'{side}.exit').read_text(encoding="utf-8")),
        'test_log_sha256': hashlib.sha256((run/f'{side}.log').read_bytes()).hexdigest(),
        'test_xml_sha256': hashlib.sha256((run/f'{side}.xml').read_bytes()).hexdigest(),
    }
assert {(case['name'], case['status']) for case in result['before']['tests']} == {
    (case['name'], case['status']) for case in result['after']['tests']}
assert result['after']['changed_tracked_files'] == [relative]
result['disposition'] = 'rejected_marginal_reduction; full publication gate not passed'
result['fixture_coverage'] = '54 CSV artifacts / 3175 rows; three existing typed report fixtures'
result['upstream_source_drift'] = [name for name, digest in snapshot['sha256'].items()
    if hashlib.sha256((Path(snapshot['source'])/name).read_bytes()).hexdigest() != digest]
result['research_program_lines'] = {path.name: len(path.read_text(encoding="utf-8").splitlines())
    for path in record.iterdir() if path.suffix in ('.py', '.ps1')}
(record/'candidate.patch').write_text(''.join(difflib.unified_diff(texts['before'].splitlines(True),
    texts['after'].splitlines(True), fromfile='a/'+relative, tofile='b/'+relative)))
(record/'result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({side: result[side]['module'] for side in ('before', 'after')}, indent=2))
print('Upstream changed files:', result['upstream_source_drift'])
