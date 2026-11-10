# Install the lesscode-core binary from a GitHub release (Windows x64).
#
#   irm https://github.com/norandom/lesscode/releases/latest/download/install.ps1 | iex
#
# Environment:
#   LESSCODE_VERSION      release to install, for example 1.0.1 (default: latest)
#   LESSCODE_INSTALL_DIR  directory for the binary
#                         (default: %LOCALAPPDATA%\Programs\lesscode-core\bin)
#   LESSCODE_ADD_TO_PATH  set to 1 to append the directory to the user PATH
#
# The script downloads one archive and its SHA-256 file, verifies the digest,
# and copies the binary and license notices. It needs no administrator rights
# and leaves PATH unchanged unless LESSCODE_ADD_TO_PATH is 1.
# Works in Windows PowerShell 5.1 and PowerShell 7.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = 'norandom/lesscode'
$version = if ($env:LESSCODE_VERSION) { $env:LESSCODE_VERSION } else { 'latest' }
$installDir = if ($env:LESSCODE_INSTALL_DIR) { $env:LESSCODE_INSTALL_DIR } else {
    Join-Path $env:LOCALAPPDATA 'Programs\lesscode-core\bin'
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'install.ps1: only 64-bit Windows is supported; build from source instead.'
}

$name = 'lesscode-core-x86_64-pc-windows-msvc'
$archive = "$name.zip"
$base = if ($version -eq 'latest') {
    "https://github.com/$repo/releases/latest/download"
} else {
    "https://github.com/$repo/releases/download/v$($version.TrimStart('v'))"
}

[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$work = Join-Path ([IO.Path]::GetTempPath()) ("lesscode-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    Write-Host "Downloading $archive ($version)"
    $zip = Join-Path $work $archive
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive" -OutFile $zip
    Invoke-WebRequest -UseBasicParsing -Uri "$base/$archive.sha256" -OutFile "$zip.sha256"

    $expected = ((Get-Content "$zip.sha256" -TotalCount 1) -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash -Algorithm SHA256 -Path $zip).Hash.ToLowerInvariant()
    if (-not $expected -or $expected -ne $actual) {
        throw "install.ps1: checksum mismatch for $archive (expected $expected, got $actual)."
    }

    Expand-Archive -Path $zip -DestinationPath $work
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    foreach ($file in 'lesscode-core.exe', 'LICENSE', 'LICENSES.md', 'THIRD_PARTY_NOTICES.txt') {
        Copy-Item -Force -Path (Join-Path $work "$name\$file") -Destination $installDir
    }
} finally {
    Remove-Item -Recurse -Force -Path $work -ErrorAction SilentlyContinue
}

$exe = Join-Path $installDir 'lesscode-core.exe'
$installed = & $exe --version
if ($LASTEXITCODE -ne 0) {
    throw 'install.ps1: the installed binary does not run on this system.'
}
Write-Host "Installed $installed to $exe"

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$onPath = @($userPath -split ';') -contains $installDir
if (-not $onPath -and $env:LESSCODE_ADD_TO_PATH -eq '1') {
    $newPath = (@($userPath -split ';' | Where-Object { $_ }) + $installDir) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Host "Added $installDir to the user PATH. Open a new terminal to use it."
} elseif (-not $onPath) {
    Write-Host "Add $installDir to PATH to run lesscode-core by name; the Git hook needs that."
}
