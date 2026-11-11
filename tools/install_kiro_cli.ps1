# Install the pinned kiro-cli release binary (openidea-kiro) into tools/bin, which is not committed.
# kiro-cli audits the specs under .kiro/specs and the code; it is part of the guard for refactoring steps
# (.kiro/specs/gradual-refactoring-steps). The digests are those of SHA256SUMS.txt of the release; a version
# change needs new digests:
#   gh release download vVERSION --repo norandom/openidea-kiro --pattern SHA256SUMS.txt
param([string]$InstallDir = (Join-Path $PSScriptRoot 'bin'))
$ErrorActionPreference = 'Stop'
$version = '1.1.0'
$asset = "kiro-cli-v$version-x86_64-pc-windows-msvc.zip"
$digest = '7edf626dae1990a00db014279922b6f1d514b94b6c56f7dda7470b713771d2bc'

$exe = Join-Path $InstallDir 'kiro-cli.exe'
if ((Test-Path $exe) -and ((& $exe --version) -eq "kiro-cli $version")) {
    Write-Output "kiro-cli $version is already in $InstallDir"
    exit 0
}
$work = Join-Path ([IO.Path]::GetTempPath()) "kiro-cli-$version-$PID"
New-Item -ItemType Directory -Force $work, $InstallDir | Out-Null
try {
    $archive = Join-Path $work $asset
    Invoke-WebRequest "https://github.com/norandom/openidea-kiro/releases/download/v$version/$asset" -OutFile $archive
    $found = (Get-FileHash $archive -Algorithm SHA256).Hash.ToLower()
    if ($found -ne $digest) { throw "install_kiro_cli: digest of $asset is $found, expected $digest" }
    Expand-Archive $archive -DestinationPath $work -Force
    Copy-Item (Get-ChildItem $work -Recurse -Filter kiro-cli.exe | Select-Object -First 1).FullName $exe -Force
    Write-Output "Installed $(& $exe --version) to $exe"
}
finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
