"""Fresh receipt for the two isolated trials; no application changes."""
import hashlib
import json
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET

docs = Path('docs/experiments')
results = {}
for name, labels in [('proxy-routing-trial', ['baseline-tests', 'candidate-tests']),
                     ('facdrone-csv-trial', ['before', 'after'])]:
    folder, run = docs/name, Path('.runs')/name
    snapshot = json.loads((folder/'snapshot.json').read_text(encoding="utf-8"))
    checks = {}
    for label in labels:
        log = run/f'{label}.log'
        xml = run/f'{label}.xml'
        suffix = '.exit' if name == 'facdrone-csv-trial' else '.exit'
        suites = ET.parse(xml).getroot().findall('testsuite')
        checks[label] = {'exit': int((run/f'{label}{suffix}').read_text(encoding="utf-8")),
                         'suites': [element.attrib for element in suites],
                         'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
                         'junit_sha256': hashlib.sha256(xml.read_bytes()).hexdigest()}
    changed = [relative for relative, digest in snapshot['sha256'].items()
               if hashlib.sha256((Path(snapshot['source'])/relative).read_bytes()).hexdigest() != digest]
    assert not changed, changed
    programs = {path.name: {'lines': len(path.read_text(encoding="utf-8").splitlines()),
                            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
                for path in folder.iterdir() if path.suffix in ('.py', '.ps1')}
    results[name] = {'checks': checks, 'specimen_tracked_files_unchanged': len(snapshot['sha256']),
                     'research_programs': programs,
                     'research_program_lines': sum(item['lines'] for item in programs.values())}
results['uv'] = subprocess.check_output(['uv.exe', '--version']).decode().strip()
results['review'] = 'inline; no independent code review claimed'
results['smt'] = 'not run'
results['native_lesscode_compare'] = 'not run; raw AST/full-module accounting and paired external checks'
(docs/'proxy-routing-trial/verification.json').write_text(json.dumps(results, indent=2)+'\n')
print(json.dumps({name: {'unchanged': item['specimen_tracked_files_unchanged'],
                          'research_lines': item['research_program_lines']}
                  for name, item in results.items() if isinstance(item, dict)}, indent=2))
