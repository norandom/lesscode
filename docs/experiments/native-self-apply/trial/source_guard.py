"""Frozen external pytest provenance guard, loaded as check-root conftest.py."""
import hashlib
import json
import os
from pathlib import Path


def _output(config, suffix, value):
    worker = getattr(config, 'workerinput', {}).get('workerid', 'controller')
    directory = Path(os.environ['NS01_ORIGIN_DIRECTORY'])
    directory.mkdir(exist_ok=True, parents=True)
    (directory/(worker+'-'+suffix+'.json')).write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')


def pytest_sessionstart(session):
    import lesscode_baseline
    from lesscode_baseline import contracts
    from lesscode_baseline.rules import contracts as rules
    expected = Path(os.environ['NS01_EXPECTED_SRC']).resolve()
    expected_hashes = json.loads(os.environ['NS01_EXPECTED_MODULE_HASHES'])
    observed = {}
    for module, relative in ((lesscode_baseline,'lesscode_baseline/__init__.py'),
                             (contracts,'lesscode_baseline/contracts.py'),
                             (rules,'lesscode_baseline/rules/contracts.py')):
        path = Path(module.__file__).resolve()
        assert path == expected/relative, f'Wrong tested implementation: {path}; expected {expected/relative}'
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        assert digest == expected_hashes[relative], f'Tested implementation changed: {path}'
        observed[relative] = dict(path=str(path),sha256=digest)
    _output(session.config,'origin',dict(expected_src=str(expected),observed=observed,pid=os.getpid()))


def pytest_collection_finish(session):
    items = sorted(item.nodeid for item in session.items)
    if items:
        assert len(items)==515, f'Expected515 unchanged test identities, collected {len(items)}'
        _output(session.config,'collection',dict(count=len(items),nodeids=items))
