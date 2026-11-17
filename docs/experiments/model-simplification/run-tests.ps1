param([int]$LimitSeconds = 1200, [switch]$WithoutPins)
$ErrorActionPreference = 'Stop'
$caseRoot = (Get-Location).Path
$allowedRoot = 'D:\BuildArtifacts\lesscode\model-simplification\ms1\'
if (-not $caseRoot.StartsWith($allowedRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Tests must run from an isolated experiment tree'
}
if ($LimitSeconds -lt 1 -or $LimitSeconds -gt 1200) { throw 'Invalid time limit' }
$runId = [Guid]::NewGuid().ToString('N')
$runRoot = Join-Path $allowedRoot "test-runs\$runId"
New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
$archive = Join-Path $runRoot 'input.tar'
$container = "lesscode-ms1-$runId"
$image = 'c2d586c24d16ae0e9d614d53be8bdb50fab61ad83a0a4af40abf63c510473a64'
function Checked([string]$Program, [string[]]$Arguments) {
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Program failed ($LASTEXITCODE)" }
}
$created = $false
$result = 125
try {
    Checked tar.exe @('-cf', $archive, '--exclude=.git', '--exclude=.venv',
        '--exclude=__pycache__', '--exclude=.pytest_cache', '-C', $caseRoot, '.')
    Checked podman @('create', '--name', $container, '--network', 'none', '--user', '65534:65534',
        '--workdir', '/case', $image, 'sleep', 'infinity')
    $created = $true
    New-Item -ItemType Directory -Path "$runRoot\case" | Out-Null
    Checked podman @('cp', "$runRoot\case", "${container}:/case")
    Checked podman @('cp', $archive, "${container}:/tmp/case.tar")
    Checked podman @('start', $container)
    Checked podman @('exec', '--user', '0', $container, 'tar', '-xf', '/tmp/case.tar', '-C', '/case')
    Checked podman @('exec', '--user', '0', $container, 'chown', '-R', '0:0', '/case')
    Checked podman @('exec', '--user', '0', $container, 'chmod', '-R', 'a-w', '/case')
    $tests = @('tests/test_exporter_resume.py', 'tests/mcp/test_tools.py')
    if (-not $WithoutPins) { $tests += 'ms1_pins' }
    & podman exec --env PYTHONPATH=/case --env PYTHONDONTWRITEBYTECODE=1 --env HOME=/tmp $container timeout --signal=KILL $LimitSeconds `
        /work/.venv/bin/python -m pytest -n 2 -q -p no:cacheprovider -c /case/pyproject.toml `
        --rootdir /case -o pythonpath=/case @tests 2>&1 |
        Tee-Object -FilePath "$runRoot\pytest.log"
    $result = $LASTEXITCODE
    if ($result -in @(125, 126, 127)) { [Console]::Error.WriteLine('MS1_INFRA_FAILURE') }
} catch {
    [Console]::Error.WriteLine('MS1_INFRA_FAILURE')
    Write-Error $_ -ErrorAction Continue
} finally {
    if ($created) { & podman rm --force $container | Out-Null }
}
exit $result
