"""Freeze tracked working-tree files; never modify the specimen."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

source = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('D:/Source/mvibe')
run = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('D:/Source/lesscode/.runs/proxy-routing-trial')
record = Path(sys.argv[3]) if len(sys.argv) > 3 else Path(__file__).parent
run.mkdir(parents=True, exist_ok=True)
files = subprocess.check_output(['git', '-C', str(source), 'ls-files', '-z']).decode().split('\0')
hashes = {}
for name in filter(None, files):
    original = source / name
    if not original.is_file():
        continue
    if original.is_symlink():
        raise ValueError(name)
    raw = original.read_bytes()
    hashes[name] = hashlib.sha256(raw).hexdigest()
    for side in ('before', 'after'):
        target = run / side / name
        if target.exists():
            raise FileExistsError(target)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
result = {
    'source': str(source),
    'revision': subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD']).decode().strip(),
    'status': subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain']).decode(),
    'sha256': hashes,
    'hypothesis_sha256': hashlib.sha256((record / 'hypothesis.md').read_bytes()).hexdigest(),
}
(record / 'snapshot.json').write_text(json.dumps(result, indent=2) + '\n')
print(f'Frozen {len(hashes)} tracked files in each isolated copy.')
