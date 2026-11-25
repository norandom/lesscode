# Prototype of the whole check on one real write-off: which commit last changed the
# entry, whether it carries an SSH signature that verifies, and whether its key is
# both in the signer list and among the forge's signing keys.
param([string]$Repo, [string]$File, [string]$Fault, [string]$SignerList, [string]$ForgeJson, [string]$Work)
$ErrorActionPreference = 'Stop'
Set-Location $Repo
$sk = 'ssh-keygen'

function Entry([string]$rev) {
    $text = git show "${rev}:$File" 2>$null
    if ($LASTEXITCODE -ne 0) { return '' }
    $j = ($text -join "`n") | ConvertFrom-Json
    $e = $j.write_offs | Where-Object { $_.fault -eq $Fault }
    if ($null -eq $e) { return '' }
    return ($e | ConvertTo-Json -Compress -Depth 5)
}

# the newest commit whose entry differs from its first parent's
$changer = $null
foreach ($c in (git log --format=%H -- $File)) {
    $parent = (git rev-parse "$c^1" 2>$null)
    if ((Entry $c) -ne (Entry $parent)) { $changer = $c; break }
}
"entry $Fault last changed by: $(git log -1 --format='%h %s' $changer)"
"marked approved: $((Entry $changer) -match '""approved"":true')"

# payload and signature, with LF exactly as stored
$raw = (git cat-file commit $changer) ; $payload = @(); $sig = @(); $h = 0; $s = 0
foreach ($l in $raw) {
    if ($h -eq 0 -and $l -eq '') { $h = 1 }
    if ($h -eq 0 -and $l.StartsWith('gpgsig ')) { $s = 1; $sig += $l.Substring(7); continue }
    if ($h -eq 0 -and $s -eq 1 -and $l.StartsWith(' ')) { $sig += $l.Substring(1); continue }
    $s = 0; $payload += $l
}
if ($sig.Count -eq 0) { "verdict: does not count - the commit is not signed"; exit 1 }
[IO.File]::WriteAllText("$Work\payload", ($payload -join "`n") + "`n")
[IO.File]::WriteAllText("$Work\sig", ($sig -join "`n") + "`n")
$p = Start-Process $sk -ArgumentList '-Y', 'check-novalidate', '-n', 'git', '-s', "`"$Work\sig`"" -RedirectStandardInput "$Work\payload" -RedirectStandardOutput "$Work\out" -RedirectStandardError "$Work\err" -NoNewWindow -Wait -PassThru
$said = (Get-Content "$Work\out", "$Work\err") -join ' '
if ($p.ExitCode -ne 0) { "verdict: does not count - the signature does not verify ($said)"; exit 1 }
$fp = [regex]::Match($said, 'SHA256:[A-Za-z0-9+/]+').Value
"signature verifies, key $fp"

function Fingerprints([string[]]$keys) {
    foreach ($k in $keys) {
        [IO.File]::WriteAllText("$Work\k.pub", "$k`n")
        $o = & $sk -l -f "$Work\k.pub"
        [regex]::Match(($o -join ' '), 'SHA256:[A-Za-z0-9+/]+').Value
    }
}
$listed = Fingerprints (Get-Content $SignerList | Where-Object { $_ -and -not $_.StartsWith('#') } | ForEach-Object { ($_ -split ' ')[2..3] -join ' ' })
$forge = Get-Content $ForgeJson -Raw | ConvertFrom-Json
$onForge = Fingerprints ($forge | ForEach-Object { $_.key })
"signer list: $($listed.Count) key(s); forge: $($onForge.Count) key(s)"
if ($listed -notcontains $fp) { "verdict: does not count - the key $fp is not in the signer list"; exit 1 }
if ($onForge -notcontains $fp) { "verdict: does not count - the key $fp is not among the account's signing keys on the forge"; exit 1 }
$title = ($forge | Where-Object { (Fingerprints @($_.key)) -eq $fp }).title
"verdict: counts - signed by '$title' $fp in $(git log -1 --format='%h on %cI' $changer)"
exit 0
