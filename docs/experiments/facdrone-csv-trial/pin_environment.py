"""Align the reduced reporting test environment to the specimen's uv lock."""
import importlib.metadata
from pathlib import Path
import tomllib

lock = tomllib.loads(Path('.runs/facdrone-csv-trial/before/uv.lock').read_text(encoding="utf-8"))
versions = {item['name']: item['version'] for item in lock['package']}
requirements = []
for distribution in importlib.metadata.distributions():
    name = distribution.metadata['Name'].lower().replace('_', '-')
    if name not in versions:
        raise ValueError(f'Unrecorded dependency: {name}')
    requirements.append(f'{name}=={versions[name]}')
Path('docs/experiments/facdrone-csv-trial/requirements.txt').write_text('\n'.join(sorted(requirements)) + '\n')
