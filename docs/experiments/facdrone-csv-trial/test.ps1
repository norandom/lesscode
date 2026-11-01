param([ValidateSet('before','after')][string]$Side = 'before')
$ErrorActionPreference = 'Stop'
$runRoot = 'D:/Source/lesscode/.runs/facdrone-csv-trial'
$trialRoot = "$runRoot/$Side"
$env:PYTHONPATH=($trialRoot+'/src;'+((Get-ChildItem ($trialRoot+'/packages') -Directory).FullName -join ';')+';'+$trialRoot+'/packages/facdrone-reporting/tests')
$env:FACDRONE_PROJECT_ROOT=$trialRoot
$env:OPENBLAS_NUM_THREADS='1'
$env:OPENBLAS_CORETYPE='Haswell'
Push-Location $trialRoot
try {
    uv.exe run --no-project --python "$runRoot/env/Scripts/python.exe" python -m pytest --noconftest -c packages/facdrone-reporting/pyproject.toml -m 'not port_baseline and not live_broker' -n 2 -q packages/facdrone-reporting/tests/test_artifact_publication.py packages/facdrone-reporting/tests/test_holdings_artifacts.py packages/facdrone-reporting/tests/test_usd_reporting_publication.py packages/facdrone-reporting/tests/test_release1_acceptance.py "--junitxml=$runRoot/$Side.xml" *> "$runRoot/$Side.log"
    $LASTEXITCODE | Set-Content "$runRoot/$Side.exit"
} finally { Pop-Location }
