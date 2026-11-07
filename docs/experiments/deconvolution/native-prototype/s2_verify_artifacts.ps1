# S2(a): verify frozen artifact bindings for the deconvolution-native-core prototype.
# Evidence collector for prototype-plan.md section 2 (U1 artifact admission).
# Hash bytes as stored; distinguish declared LF-body normalization hashes (Req 2.3).
# Fail-closed: missing bytes or missing sidecars are recorded as unavailable, never zero.
param(
    [string]$RepoRoot = "D:\Source\lesscode",
    [string]$OutDir = "docs\experiments\deconvolution\native-prototype\results"
)
$ErrorActionPreference = "Stop"
Set-Location $RepoRoot

function Get-FileSha256([string]$Path) {
    $h = [System.Security.Cryptography.SHA256]::Create()
    try {
        $fs = [System.IO.File]::OpenRead($Path)
        try { ($h.ComputeHash($fs) | ForEach-Object { $_.ToString("x2") }) -join "" }
        finally { $fs.Dispose() }
    } finally { $h.Dispose() }
}

function Read-Sidecar([string]$SidecarPath, [string]$ArtifactName) {
    # Returns @{ stored = hex|null; declared_lf_body = hex|null }
    if (-not (Test-Path $SidecarPath)) { return $null }
    $stored = $null; $lfBody = $null
    foreach ($line in Get-Content -LiteralPath $SidecarPath) {
        if ($line -match '^([0-9a-f]{64})\s\s(.+)$') {
            $stored = $Matches[1]   # sha256 of the file as stored
        } elseif ($line -match '^#\s*lf-body sha256 ([0-9a-f]{64})') {
            $lfBody = $Matches[1]   # declared LF-body identity (Python record())
        }
    }
    return @{ stored = $stored; declared_lf_body = $lfBody }
}

function Test-Binding([string]$ArtifactRel, [string[]]$SidecarRels) {
    $artifact = Join-Path $RepoRoot $ArtifactRel
    $entry = [ordered]@{
        artifact = $ArtifactRel
        exists = (Test-Path $ArtifactRel.Replace('/', '\'))
        bytes = $null
        sha256_stored = $null
        sidecars = @()
        status = "missing"
    }
    if (-not $entry.exists) { return $entry }
    $abs = $ArtifactRel -replace '/', '\'
    $entry.bytes = (Get-Item -LiteralPath $abs).Length
    $entry.sha256_stored = Get-FileSha256 $abs
    $entry.status = "no-sidecar"
    $anyMatch = $false; $anyMismatch = $false
    foreach ($sc in $SidecarRels) {
        $scPath = $sc -replace '/', '\'
        $info = Read-Sidecar $scPath $null
        $sidecarEntry = [ordered]@{ sidecar = $sc; exists = (Test-Path $scPath) }
        if ($null -eq $info) {
            $sidecarEntry.expected_stored = $null
            $sidecarEntry.match = "sidecar-missing"
        } else {
            $sidecarEntry.expected_stored = $info.stored
            $sidecarEntry.declared_lf_body = $info.declared_lf_body
            if ($null -eq $info.stored) { $sidecarEntry.match = "sidecar-unparseable" }
            elseif ($info.stored -eq $entry.sha256_stored) { $sidecarEntry.match = "verified"; $anyMatch = $true }
            else { $sidecarEntry.match = "mismatch"; $anyMismatch = $true }
        }
        $entry.sidecars += [pscustomobject]$sidecarEntry
    }
    if ($anyMismatch) { $entry.status = "mismatch" }
    elseif ($anyMatch) { $entry.status = "verified" }
    else { $entry.status = "no-bound-sidecar" }
    return $entry
}

function New-Manifest([string]$RootRel) {
    $root = Join-Path $RepoRoot ($RootRel -replace '/', '\')
    $files = @()
    if (Test-Path $root) {
        $files = Get-ChildItem -LiteralPath $root -Recurse -File | Sort-Object FullName | ForEach-Object {
            $rel = $_.FullName.Substring($RepoRoot.Length + 1).Replace('\', '/')
            [pscustomobject]@{ path = $rel; bytes = $_.Length; sha256_stored = (Get-FileSha256 $_.FullName) }
        }
    }
    return [pscustomobject]@{ root = $RootRel; exists = (Test-Path $root); file_count = @($files).Count; files = $files }
}

$bindings = @(
    (Test-Binding ".runs/e4-lifecycle/golden-a.json" @(
        ".runs/e4-lifecycle/golden-a.json.sha256",
        "docs/experiments/deconvolution/e4-lifecycle/results/golden-e4.json.sha256")),
    (Test-Binding ".runs/e4-lifecycle/golden-b.json" @(
        ".runs/e4-lifecycle/golden-b.json.sha256",
        "docs/experiments/deconvolution/e4-lifecycle/results/golden-e4.json.sha256")),
    (Test-Binding ".runs/e4-lifecycle/held-out-a.json" @(
        ".runs/e4-lifecycle/held-out-a.json.sha256",
        "docs/experiments/deconvolution/e4-lifecycle/results/held-out-e4.json.sha256")),
    (Test-Binding ".runs/e4-lifecycle/held-out-b.json" @(
        ".runs/e4-lifecycle/held-out-b.json.sha256",
        "docs/experiments/deconvolution/e4-lifecycle/results/held-out-e4.json.sha256")),
    (Test-Binding ".runs/e4-lifecycle/smoke.json" @(
        ".runs/e4-lifecycle/smoke.json.sha256")),
    (Test-Binding ".runs/e4-lifecycle/e4-orig.sqlite" @()),
    (Test-Binding ".runs/e4-lifecycle/e4-cand.sqlite" @()),
    (Test-Binding ".runs/e3-facdrone/t31.sqlite" @()),
    (Test-Binding ".runs/e3-facdrone/t4.sqlite" @())
)

# Frozen regenerator turns (pure turn-transformation checks bind to these bytes).
$turnRoot = "docs/experiments/deconvolution/regenerator/results/turns"
$turnDirs = @("e4-turn1", "e4-turn2", "e4-turn3", "t4-turn1", "t4-turn1-retry", "t4-turn2")
foreach ($td in $turnDirs) {
    foreach ($f in @("response.json", "user.txt")) {
        $art = "$turnRoot/$td/$f"
        $sidecars = @("$turnRoot/$td/$f.sha256")
        $bindings += Test-Binding $art $sidecars
    }
}

$manifests = @(
    (New-Manifest ".runs/e3-facdrone/cand"),
    (New-Manifest ".runs/e3-facdrone/orig-check")
)

$verified = @($bindings | Where-Object { $_.status -eq "verified" }).Count
$mismatch = @($bindings | Where-Object { $_.status -eq "mismatch" }).Count
$missing = @($bindings | Where-Object { $_.status -eq "missing" }).Count
$unbound = @($bindings | Where-Object { $_.status -in @("no-sidecar", "no-bound-sidecar") }).Count

$receipt = [ordered]@{
    receipt = "s2-artifact-admission-1"
    purpose = "S2(a) verify frozen artifact bindings (prototype-plan.md section 2, U1)"
    generated_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    host = $env:COMPUTERNAME
    command = "powershell -File docs/experiments/deconvolution/native-prototype/s2_verify_artifacts.ps1"
    hashing = "SHA256 over bytes as stored (CRLF where applicable); declared_lf_body recorded separately"
    bindings = $bindings
    manifests = $manifests
    summary = [ordered]@{
        total = @($bindings).Count
        verified = $verified
        mismatch = $mismatch
        missing_bytes = $missing
        unbound_no_sidecar = $unbound
    }
}

$outDirAbs = Join-Path $RepoRoot $OutDir
New-Item -ItemType Directory -Force -Path $outDirAbs | Out-Null
$outPath = Join-Path $outDirAbs "s2-artifact-admission.json"
$receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $outPath -Encoding utf8
$hash = Get-FileSha256 $outPath
"$hash  s2-artifact-admission.json" | Set-Content -LiteralPath "$outPath.sha256" -Encoding utf8

Write-Output "receipt: $outPath"
Write-Output ("summary: total={0} verified={1} mismatch={2} missing={3} unbound={4}" -f @($bindings).Count, $verified, $mismatch, $missing, $unbound)
foreach ($b in $bindings) { Write-Output ("{0,-70} {1}" -f $b.artifact, $b.status) }
if ($mismatch -gt 0) { exit 1 } else { exit 0 }
