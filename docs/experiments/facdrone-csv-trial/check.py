"""Compare actual normalized CSV bytes from existing typed report fixtures."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

run = Path('.runs/facdrone-csv-trial').resolve()
before, after = run / 'before', run / 'after'
sys.path[:0] = [str(before / 'src'), *(str(path) for path in (before / 'packages').iterdir() if path.is_dir()),
               str(before / 'packages/facdrone-reporting/tests')]
import os
os.environ['FACDRONE_PROJECT_ROOT'] = str(before)
from facdrone_reporting.testing.artifact_publication import small_result
from facdrone_reporting.testing.usd_reporting_support import usd_result
from test_run_report_builder import _flagship
from facdrone_reporting.library import normalized_csv as original
from facdrone_reporting.shells.core import _render_csv
from facdrone_reporting.declarations.metric_declarations import summary_metric_membership

relative = 'packages/facdrone-reporting/facdrone_reporting/library/normalized_csv.py'
spec = importlib.util.spec_from_file_location('candidate_csv', after / relative)
candidate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(candidate)
observations = []
for name, fixture in [('small_eur', small_result), ('flagship_eur', _flagship), ('usd', usd_result)]:
    report = fixture()
    old = original.project_normalized_csv_bundle(report, summary_metric_membership())
    new = candidate.project_normalized_csv_bundle(report, summary_metric_membership())
    assert len(old) == len(new) == 18
    for left, right in zip(old, new):
        old_artifact, new_artifact = _render_csv(left, report), _render_csv(right, report)
        assert old_artifact == new_artifact
        observations.append({'fixture': name, 'file': old_artifact.filename, 'rows': len(left.rows),
                             'sha256': hashlib.sha256(old_artifact.content).hexdigest()})

# Wrong-field mutation must break a real output, not merely a helper assertion.
field = candidate._field
candidate._field = lambda value, name: 'MUTATED' if name == 'portfolio_return' else field(value, name)
report = small_result()
mutation_killed = original.project_aligned_returns(report) != candidate.project_aligned_returns(report)
assert mutation_killed
candidate._field = field

# Preserve whole-object validation: even an unrelated invalid field must fail.
from dataclasses import dataclass
@dataclass
class BadRecord:
    selected: str = 'valid'
    unrelated: object = object()
def error(function):
    try:
        function()
    except Exception as caught:
        return type(caught).__name__, str(caught)
    raise AssertionError('Invalid record unexpectedly accepted')
assert error(lambda: original._field(BadRecord(), 'selected')) == error(
    lambda: candidate._fields(BadRecord(), 'selected'))
result = {'tables': observations, 'mismatches': 0, 'wrong_field_mutation_killed': mutation_killed,
          'unrelated_invalid_field_rejection_preserved': True, 'python': sys.version,
          'solver': 'not used; unsupported canonical serialization semantics'}
Path('docs/experiments/facdrone-csv-trial/oracle.json').write_text(json.dumps(result, indent=2) + '\n')
print(f'{len(observations)} exact CSV artifacts; {sum(item["rows"] for item in observations)} rows; mutation caught')
