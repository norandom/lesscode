"""New user-authorized parallel-check trial; prior freezes/results retained."""
from concurrent.futures import ThreadPoolExecutor
import importlib.metadata
import json
from pathlib import Path
import shutil

import conduct
from conduct import ROOT, RUN, EXT, HERE, sha, dump, execute, uv

TRIAL = EXT/'parallel-trial-v3'


def freeze():
    TRIAL.mkdir(exist_ok=False)
    previous = EXT/'reviewed-trial-v2'
    for name in ('expected_runner.py','expected-outcomes.json','inventory.json'):
        shutil.copyfile(previous/name,TRIAL/name)
    shutil.copyfile(ROOT/'requirements-test.lock',TRIAL/'requirements-test.lock')
    shutil.copyfile(HERE/'source_guard.py',TRIAL/'source_guard.py')
    definition = json.loads((previous/'check-definition.json').read_text())
    definition['commands']['project_regressions'] = ['python','-m','pytest','-c','<execution-tree>/pyproject.toml',
        '--rootdir','<execution-tree>','-o','pythonpath=<execution-tree>/src','tests','-n','4','--dist','loadfile','-ra',
        '--durations=10','-p','no:cacheprovider','--junitxml','<absolute-external-side-xml>']
    definition['execution_tree_recipe'] += ' Copy source-snapshot root .gitattributes into execution root; verify frozen checks/tools and source hashes before and after.'
    definition['parallel_runner'] = dict(pytest=importlib.metadata.version('pytest'),
        xdist=importlib.metadata.version('pytest-xdist'), workers_per_side=4,
        test_lock_sha256=sha(TRIAL/'requirements-test.lock'),
        gitattributes_sha256=sha(RUN/'baseline/.gitattributes'),
        pytest_config_sha256=sha(RUN/'baseline/pyproject.toml'),
        source_guard_sha256=sha(TRIAL/'source_guard.py'),
        source_guard='Copied as check-root conftest.py; every controller/worker must assert selected package/basecontracts/rulescontracts paths and hashes; collection must match515 identities',
        expected_test_identities=515,
        authorization='Explicit user request for pytest-xdist and Rust nextest; controller verified unittest/pytest collection identity counters match515.')
    dump(TRIAL/'check-definition.json',definition)
    manifest = json.loads((previous/'manifest.json').read_text())
    manifest['trial_id'] += '-parallel-v3'
    for check in manifest['checks'].values():
        check['definition_digest']=sha(TRIAL/'check-definition.json')
    manifest['checks']['project_regressions']['description']='All515 existing unittest tests collected by pinned pytest-xdist, four loadfile workers per side'
    dump(TRIAL/'manifest.json',manifest)
    conduct.TRIAL=TRIAL
    execute('parallel-v3-prepare',[str(EXT/'lesscode-core.exe'),'prepare','--manifest',str(TRIAL/'manifest.json'),
        '--baseline',str(RUN/'baseline'),'--artifact-root',str(TRIAL),'--out',str(TRIAL/'frozen.json')],ROOT)
    for name in ('manifest.json','frozen.json','check-definition.json','requirements-test.lock'):
        shutil.copyfile(TRIAL/name,HERE/('parallel-v3-'+name))


def side_checks(side):
    tree=EXT/'check-runs'/('parallel-v3-'+side)
    shutil.copytree(EXT/'frozen-checks',tree)
    shutil.copytree(RUN/side/'src',tree/'src')
    shutil.copytree(ROOT/'.tools/analyzers',tree/'.tools/analyzers')
    shutil.copyfile(RUN/'baseline/.gitattributes',tree/'.gitattributes')
    shutil.copyfile(TRIAL/'source_guard.py',tree/'conftest.py')
    definition=json.loads((TRIAL/'check-definition.json').read_text())
    tools=json.loads((ROOT/'docs/experiments/native-self-apply/tool-inventory.json').read_text())
    before_checks={p:sha(tree/p) for p in definition['frozen_test_files']}
    before_tools={p:sha(tree/'.tools/analyzers'/p) for p in tools}
    assert before_checks==definition['frozen_test_files'] and before_tools==tools
    dump(HERE/('parallel-v3-'+side+'-preflight.json'),dict(frozen_file_sha256=sha(TRIAL/'frozen.json'),
        tests=before_checks,tools_inventory_sha256=sha(ROOT/'docs/experiments/native-self-apply/tool-inventory.json'),
        copied_tools_match_inventory=True,gitattributes_sha256=sha(tree/'.gitattributes')))
    modules=('lesscode_baseline/__init__.py','lesscode_baseline/contracts.py','lesscode_baseline/rules/contracts.py')
    environment=dict(NS01_EXPECTED_SRC=str(tree/'src'), NS01_EXPECTED_MODULE_HASHES=json.dumps({p:sha(tree/'src'/p) for p in modules}))
    configuration=('-c',str(tree/'pyproject.toml'),'--rootdir',str(tree),'-o','pythonpath='+str(tree/'src'))
    preflight_env=dict(environment,NS01_ORIGIN_DIRECTORY=str(TRIAL/(side+'-preflight-origin')))
    result=execute('parallel-v3-'+side+'-collection',uv('-m','pytest',*configuration,'tests','-n','0','--collect-only','-q','-p','no:cacheprovider'),
        tree,tree,extra_env=preflight_env)
    assert result['exit_code']==0
    execute('parallel-v3-'+side+'-import',uv('-c','import lesscode_baseline; print(lesscode_baseline.__file__)'),tree,tree)
    execute('parallel-v3-'+side+'-independent',uv(str(TRIAL/'expected_runner.py')),tree,tree)
    execute('parallel-v3-'+side+'-relevant',uv('-m','unittest','tests.test_contracts','tests.test_rule_contracts',
        'tests.test_rule_evidence','tests.test_rule_evaluation','-q'),tree,tree)
    execute('parallel-v3-'+side+'-regressions',uv('-m','pytest',*configuration,'tests','-n','4','--dist','loadfile','-ra',
        '--durations=10','-p','no:cacheprovider','--junitxml',str(TRIAL/(side+'-junit.xml'))),tree,tree,
        extra_env=dict(environment,NS01_ORIGIN_DIRECTORY=str(TRIAL/(side+'-origin'))))
    shutil.copyfile(TRIAL/(side+'-junit.xml'),HERE/('parallel-v3-'+side+'-junit.xml'))
    for suffix in ('origin','preflight-origin'):
        shutil.copytree(TRIAL/(side+'-'+suffix),HERE/('parallel-v3-'+side+'-'+suffix))


if __name__=='__main__':
    freeze()
    with ThreadPoolExecutor(max_workers=2) as pool:
        list(pool.map(side_checks,('baseline','candidate')))
