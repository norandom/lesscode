param(
    [Parameter(Mandatory)][string]$Tracer,
    [Parameter(Mandatory)][string]$NewEvidence,
    [string]$Frozen = 'D:/BuildArtifacts/lesscode/corrective-audit/task67'
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$exe=(Resolve-Path -LiteralPath $Tracer).Path
$root=[IO.Path]::GetFullPath($NewEvidence)
if (!$root.StartsWith('D:\BuildArtifacts\',[StringComparison]::OrdinalIgnoreCase) -or (Test-Path $root)) {
    throw 'Choose a new D:\BuildArtifacts evidence directory'
}
$target=(Resolve-Path -LiteralPath $env:CARGO_TARGET_DIR).Path
if ($target -ne (Resolve-Path 'D:/BuildArtifacts/cargo-target').Path) {throw 'Expected the compatible shared Cargo target'}
$pins=@{'pre-fix.zip'='2FACB5B3BD27585958DCE696B914414AE005215193C2CF10AE3076D50740F96A';
    'fix.zip'='4D4C495C24D451810FC09333785475F24CFE2EDC97518328073FC271DDD2B3BF';
    'toy-check.exe'='B0501621C25D17C866D911ACF2E8DA8BB7E6AAF1AEC6B26271E39413BB380D78';
    'pre-driver.exe'='C9A614D797908079CCFAC1EA52566C6EFFCBC9A22ED97E92000CB4B6460FD512';
    'fixed-verified-driver.exe'='F04FC2719CB34E40EFF30C54D324B4A475B07FBE9E1F0EC8EE37468A9010F56C'}
foreach ($name in $pins.Keys) {
    if ((Get-FileHash -LiteralPath "$Frozen/$name").Hash -ne $pins[$name]) {throw "Frozen input drift at $name"}
}
$harness=Join-Path $PSScriptRoot '../protocol-review/replay-history.ps1'
$null=New-Item -ItemType Directory -Path $root
foreach ($arm in 'before','fixed','disabled') {
    $dst="$root/$arm"
    $archive=if ($arm -eq 'before') {'pre-fix.zip'} else {'fix.zip'}
    Expand-Archive -LiteralPath "$Frozen/$archive" -DestinationPath "$dst/build"
    $native="$dst/build/native"
    if ($arm -eq 'disabled') {
        $runner="$native/src/adapters/driver/runner.rs"
        $s=[IO.File]::ReadAllText($runner)
        $guard='if sync::refresh(&home, &config.source_root, &config.exclusions)? > 0 {'
        if($s.Split([string[]]@($guard),[StringSplitOptions]::None).Count -ne 2){throw 'Expected one fixed guard'}
        [IO.File]::WriteAllText($runner,$s.Replace($guard,$guard.Replace('if ','if false && ')))
    }
    & $exe overlay $native "$dst/overlay"
    if ($LASTEXITCODE) {throw "Extraction failed on $arm"}
    $map=Get-Content "$dst/overlay/sites.json" -Raw | ConvertFrom-Json -AsHashtable
    foreach ($file in $map.sources.Keys) {
        if((Get-FileHash -LiteralPath "$native/$file").Hash -ne $map.sources[$file]) {throw "Source changed at $file"}
    }
    foreach ($site in $map.sites) {
        $bytes=[IO.File]::ReadAllBytes("$native/$($site.file)")
        $quote=[Text.Encoding]::UTF8.GetString($bytes,$site.bytes.start,$site.bytes.end-$site.bytes.start)
        $line=1+[regex]::Matches([Text.Encoding]::UTF8.GetString($bytes,0,$site.bytes.start),"`n").Count
        if ($quote -cne $site.quote -or $line -ne $site.start_line) {throw "Bad source citation $($site.id)"}
    }
    foreach ($file in Get-ChildItem "$dst/overlay/src" -Recurse -File) {
        $relative=[IO.Path]::GetRelativePath("$dst/overlay",$file.FullName)
        Copy-Item -LiteralPath $file.FullName -Destination "$native/$relative"
    }
    # Unique package IDs keep the three historical builds out of one another's cache.
    $package="lc-worker-trace-$arm"
    foreach ($file in 'Cargo.toml','Cargo.lock') {
        $path="$native/$file"
        $s=[IO.File]::ReadAllText($path).Replace('name = "lesscode-core"',('name = "'+$package+'"'))
        if($file -eq 'Cargo.toml') {$s += "`n[lib]`nname = `"lesscode_core`"`n"}
        [IO.File]::WriteAllText($path,$s)
    }
    Push-Location -LiteralPath $native
    try {
        cargo +1.98.1 build --locked --offline --bin $package *> "$dst/build.log"
        if($LASTEXITCODE){throw "Build failed on $arm; see $dst/build.log"}
        Copy-Item "$target/debug/$package.exe" "$dst/driver.exe"
    } finally {Pop-Location}
    $null=New-Item -ItemType Directory -Path "$dst/events","$dst/children"
    $settings=@{LC_WORKER_DRIVER="$dst/driver.exe"; LC_WORKER_TOY="$Frozen/toy-check.exe";
        LC_WORKER_LOGS="$dst/events"; LC_WORKER_OBSERVATIONS="$dst/children"}
    $previous=@{}
    foreach ($key in $settings.Keys) {
        $previous[$key]=[Environment]::GetEnvironmentVariable($key)
        [Environment]::SetEnvironmentVariable($key,$settings[$key])
    }
    $behavior=if ($arm -eq 'fixed') {'after'} else {'before'}
    try {
        & $harness -Driver $exe -ToyCheck $exe -NewEvidence "$dst/history" -Revision $behavior
        if($LASTEXITCODE){throw "Independent fixture assertions failed on $arm"}
    } finally {
        foreach ($key in $settings.Keys) {[Environment]::SetEnvironmentVariable($key,$previous[$key])}
    }
    & $exe predict "$dst/overlay/sites.json" "$dst/events" tests/check.txt "$dst/prediction.json"
    if($LASTEXITCODE){throw "Driver-event prediction failed on $arm"}
}
foreach ($arm in 'before','fixed') {
    $name=if ($arm -eq 'before') {'pre-driver.exe'} else {'fixed-verified-driver.exe'}
    $behavior=if ($arm -eq 'fixed') {'after'} else {'before'}
    & $harness -Driver "$Frozen/$name" -ToyCheck "$Frozen/toy-check.exe" -NewEvidence "$root/plain-$arm" -Revision $behavior
    if($LASTEXITCODE){throw "Uninstrumented comparison failed on $arm"}
}
& (Join-Path $PSScriptRoot 'verify.ps1') -Tracer $exe -Evidence $root
if($LASTEXITCODE){exit $LASTEXITCODE}
exit 0
