param([string]$OutputDirectory)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
$started = [DateTime]::UtcNow.ToString('o')
if (-not $OutputDirectory) {
    $OutputDirectory = Join-Path $repo ('.runs/r39-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfff'))
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Use a new output directory to preserve prior evidence.' }
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$run = (Resolve-Path -LiteralPath $OutputDirectory).Path
$python = Join-Path $repo '.runs/e2-facdrone/venv/Scripts/python.exe'
if (-not (Test-Path -LiteralPath $python)) { throw 'Pinned specimen Python is required; probes must not skip.' }
$env:PATH = 'C:\Windows\System32;C:\Windows;' + $env:USERPROFILE + '\.cargo\bin;C:\Program Files\Git\cmd;' + $env:LOCALAPPDATA + '\Microsoft\WinGet\Packages\astral-sh.uv_Microsoft.Winget.Source_8wekyb3d8bbwe'
$env:PYTHONPATH = Join-Path $repo 'src'
$env:R39_WORK = Join-Path $run 'cases'
$env:R39_PYTHON = $python
$env:CARGO_TARGET_DIR = Join-Path $repo 'native/target'
$native = (Join-Path $repo 'native').Replace('\', '/')
$probe = (Join-Path $PSScriptRoot 'probe.rs').Replace('\', '/')
@"
[package]
name = "r39-correctness"
version = "0.0.0"
edition = "2021"
[dependencies]
lesscode-core = { path = "$native" }
serde_json = "=1.0.154"
[[test]]
name = "correctness"
path = "$probe"
"@ | Set-Content -LiteralPath (Join-Path $run 'Cargo.toml') -Encoding utf8
Copy-Item -LiteralPath (Join-Path $repo 'native/Cargo.lock') -Destination (Join-Path $run 'Cargo.lock')
$paths = @('native/src/regenerator.rs', 'native/src/recording.rs', 'native/src/surface.rs', 'native/src/measures.rs', 'native/src/receipt.rs', 'native/src/jsonval.rs', 'native/Cargo.toml', 'native/Cargo.lock', 'src/lesscode_baseline/regenerator.py', 'src/lesscode_baseline/surface.py', 'src/lesscode_baseline/measures.py', 'docs/experiments/deconvolution/refinement-correctness/probe.rs', 'docs/experiments/deconvolution/refinement-correctness/test_reference.py', 'docs/experiments/deconvolution/refinement-correctness/run.ps1', 'docs/experiments/deconvolution/e3-evidence-regeneration/results/sql-divergences-round21.json', 'docs/experiments/deconvolution/e4-lifecycle/results/sql-divergences-round21.json', 'src/lesscode_baseline/rules/evaluate.py', 'tests/test_rule_evaluation.py')
function Snapshot {
    $values = [ordered]@{}
    foreach ($path in $paths) { $values[$path] = (Get-FileHash -LiteralPath (Join-Path $repo $path) -Algorithm SHA256).Hash.ToLowerInvariant() }
    return $values
}
$before = Snapshot
Push-Location $repo
try {
    $head = (git rev-parse HEAD).Trim()
    $rustVersion = (rustup run 1.98.1 rustc --version | Out-String).Trim()
    $nextestVersion = (rustup run 1.98.1 cargo nextest --version | Out-String).Trim()
    $pythonVersion = (& $python --version | Out-String).Trim()
    $pythonPackages = (& $python -c 'import pytest, xdist; print("pytest=" + pytest.__version__ + ", xdist=" + xdist.__version__)' | Out-String).Trim()
    $nativeStarted = [DateTime]::UtcNow.ToString('o')
    rustup run 1.98.1 cargo nextest run --offline --manifest-path (Join-Path $run 'Cargo.toml') --test-threads 4 --no-fail-fast --failure-output immediate 2>&1 | Tee-Object -FilePath (Join-Path $run 'native.log')
    $nativeExit = $LASTEXITCODE
    $nativeFinished = [DateTime]::UtcNow.ToString('o')
    $pythonStarted = [DateTime]::UtcNow.ToString('o')
    uv.exe run --no-project --no-python-downloads --python $python python -m pytest (Join-Path $PSScriptRoot 'test_reference.py') -n 4 --dist loadfile -q --tb=short 2>&1 | Tee-Object -FilePath (Join-Path $run 'reference.log')
    $pythonExit = $LASTEXITCODE
    $pythonFinished = [DateTime]::UtcNow.ToString('o')
} finally { Pop-Location }
$after = Snapshot
$changed = @($paths | Where-Object { $before[$_] -ne $after[$_] })
$report = [ordered]@{
    report = 'R39 exploratory correctness probes; red assertions are defects, not a passing feature gate'
    head = $head; started_utc = $started; finished_utc = [DateTime]::UtcNow.ToString('o')
    rustc = $rustVersion; nextest = $nextestVersion; python = $pythonVersion; python_packages = $pythonPackages
    native = @{ started_utc = $nativeStarted; finished_utc = $nativeFinished; exit_code = $nativeExit }
    reference = @{ started_utc = $pythonStarted; finished_utc = $pythonFinished; exit_code = $pythonExit }
    source_and_probe_sha256 = $before; changed_during_run = $changed
    resolved_lock_sha256 = (Get-FileHash -LiteralPath (Join-Path $run 'Cargo.lock')).Hash.ToLowerInvariant()
    native_log_sha256 = (Get-FileHash -LiteralPath (Join-Path $run 'native.log')).Hash.ToLowerInvariant()
    reference_log_sha256 = (Get-FileHash -LiteralPath (Join-Path $run 'reference.log')).Hash.ToLowerInvariant()
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $run 'provenance.json') -Encoding utf8
if ($changed.Count) { throw ('Source changed during probes: ' + ($changed -join ', ')) }
if ($nativeExit -ne 0 -or $pythonExit -ne 0) { exit 1 }
exit 0
