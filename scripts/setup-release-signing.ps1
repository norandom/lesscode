<#
.SYNOPSIS
Sets up release signing once (docs/release-signing.md, steps 1 to 3).

.DESCRIPTION
Run it from the repository root on your own machine:

    powershell -ExecutionPolicy Bypass -File scripts\setup-release-signing.ps1

It is safe to run again. Each step is skipped when it is already done:
  1. installs zipsign 0.2.2 and creates a key pair in ~\lesscode-release-key
     (an existing key is never replaced). This key is separate from the
     mmhmem release key;
  2. creates the GitHub 'release' environment with you as the required
     reviewer and stores the private key as its secret
     LESSCODE_RELEASE_SIGNING_KEY (the key is never printed);
  3. adds the public key to scripts/release-public-keys.txt.

Afterwards, commit scripts/release-public-keys.txt and push it to main. Keep a
backup of ~\lesscode-release-key\release.key in a password manager or offline.

Needs: gh (logged in with admin rights on the repository) and cargo.
#>
param(
    [string]$Repo = 'norandom/lesscode',
    [string]$KeyDir = (Join-Path $HOME 'lesscode-release-key')
)
$ErrorActionPreference = 'Stop'

$ZipsignVersion = '0.2.2'
$SecretName = 'LESSCODE_RELEASE_SIGNING_KEY'
$KeyFile = Join-Path $KeyDir 'release.key'
$PubFile = Join-Path $KeyDir 'release.pub'
$KeyList = Join-Path $PSScriptRoot 'release-public-keys.txt'

function Step([string]$Text) { Write-Host ''; Write-Host "== $Text" -ForegroundColor Cyan }
function Done([string]$Text) { Write-Host "   $Text" -ForegroundColor Green }

function Invoke-Native([string]$What, [scriptblock]$Command) {
    $output = & $Command
    if ($LASTEXITCODE -ne 0) { throw "$What failed (exit code $LASTEXITCODE)." }
    $output
}

function Get-ZipsignVersion {
    if (-not (Get-Command zipsign -ErrorAction SilentlyContinue)) { return $null }
    ((& zipsign --version) -join ' ').Trim()
}

# Checks the tools first, so nothing is half done.
foreach ($tool in 'gh', 'cargo') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "'$tool' was not found. Install it and run this script again."
    }
}
& gh auth status *> $null
if ($LASTEXITCODE -ne 0) { throw "gh is not logged in. Run 'gh auth login' first." }
if (-not (Test-Path -LiteralPath $KeyList)) {
    throw "Cannot find $KeyList. Run the script from a checkout of the repository."
}

Step '1. Signing key'
if ((Get-ZipsignVersion) -ne "zipsign $ZipsignVersion") {
    Invoke-Native 'Installing zipsign' { cargo install zipsign --version "=$ZipsignVersion" --locked } | Out-Null
}
$version = Get-ZipsignVersion
if ($version -ne "zipsign $ZipsignVersion") { throw "Expected zipsign $ZipsignVersion, found '$version'." }
Done $version

$hasKey = Test-Path -LiteralPath $KeyFile
$hasPub = Test-Path -LiteralPath $PubFile
if ($hasKey -and $hasPub) {
    Done "Using the existing key in $KeyDir"
} elseif ($hasKey -or $hasPub) {
    throw "Only one of release.key and release.pub is in $KeyDir. Restore the missing file from your backup; this script never replaces a key."
} else {
    New-Item -ItemType Directory -Force -Path $KeyDir | Out-Null
    Invoke-Native 'Creating the key' { zipsign gen-key $KeyFile $PubFile } | Out-Null
    Done "Created $KeyFile and $PubFile"
}
$keyBytes = [IO.File]::ReadAllBytes($KeyFile)
$pubBytes = [IO.File]::ReadAllBytes($PubFile)
if ($keyBytes.Length -ne 64) { throw "$KeyFile should be 64 bytes, it is $($keyBytes.Length)." }
if ($pubBytes.Length -ne 32) { throw "$PubFile should be 32 bytes, it is $($pubBytes.Length)." }

Step "2. GitHub 'release' environment for $Repo"
$reviewer = (Invoke-Native 'Reading your GitHub user id' { gh api user --jq .id }).Trim()
$body = '{"reviewers":[{"type":"User","id":' + $reviewer + '}],"deployment_branch_policy":null}'
$body | gh api --method PUT "repos/$Repo/environments/release" --input - | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Creating the 'release' environment failed. You need admin rights on $Repo." }
Done "Environment 'release' with you as the required reviewer"

[Convert]::ToBase64String($keyBytes) | gh secret set $SecretName --env release --repo $Repo
if ($LASTEXITCODE -ne 0) { throw 'Storing the secret failed.' }
$secrets = Invoke-Native 'Listing the secrets' { gh secret list --env release --repo $Repo }
if (-not ($secrets -match "^$SecretName\s")) { throw 'The secret is not listed in the release environment.' }
Done "Secret $SecretName stored (value not shown)"

Step '3. Public key in scripts/release-public-keys.txt'
$pubHex = ($pubBytes | ForEach-Object { $_.ToString('x2') }) -join ''
# The publisher's own parser decides whether the key is listed.
. (Join-Path $PSScriptRoot 'sign-release-assets.ps1')
if (@(Get-ReleasePublicKeys $KeyList) -contains $pubHex) {
    Done 'Already there'
} else {
    $text = [IO.File]::ReadAllText($KeyList)
    $text = $text.Replace("# Intentionally empty until the owner generates the release key.`n", '')
    if (-not $text.EndsWith("`n")) { $text += "`n" }
    $text += "$pubHex  # added $(Get-Date -Format 'yyyy-MM-dd')`n"
    [IO.File]::WriteAllText($KeyList, $text, (New-Object Text.UTF8Encoding $false))
    Done "Added $pubHex"
}

Write-Host ''
Write-Host 'All set. Next:' -ForegroundColor Green
Write-Host '  - commit scripts/release-public-keys.txt and push it to main'
Write-Host "  - back up $KeyFile in a password manager or offline"
