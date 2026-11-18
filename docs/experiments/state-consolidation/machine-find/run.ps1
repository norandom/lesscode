# R156: the finder on the reply machine before consolidation, three controls
# and a sweep of the repository's Rust sources at two revisions.
#   ./run.ps1 -Binary <find-machine.exe> -Out <new directory>
param([Parameter(Mandatory)][string]$Binary, [Parameter(Mandatory)][string]$Out,
      [string]$Before = '1eb106f')
$ErrorActionPreference = 'Stop'
$repo = (git rev-parse --show-toplevel)
$rules = Join-Path $PSScriptRoot 'rust.state.json'
$four = 'tools/guide-optimizer/src/adapter.rs', 'model/src/repair.rs', 'model/src/check.rs', 'model/src/request.rs'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
New-Item -ItemType Directory -Force $Out | Out-Null

function Find([string]$Root, [string[]]$Files, [string]$Name) {
    & $Binary $rules $Root @Files | Set-Content (Join-Path $Out "$Name.json") -Encoding utf8NoBOM
    if ($LASTEXITCODE) { throw "${Name}: exit $LASTEXITCODE" }
}
function Edit([string]$Path, [scriptblock]$Change) {
    Set-Content $Path (& $Change (Get-Content $Path -Raw)) -NoNewline -Encoding utf8NoBOM
}

# The tree before consolidation, and the four files of the case.
$tree = Join-Path $Out "tree-$Before"
New-Item -ItemType Directory -Force $tree | Out-Null
git -C $repo archive $Before native/src model/src tools | tar -x -C $tree
if ($LASTEXITCODE) { throw 'archive failed' }
Find $tree $four 'before'
Find $repo $four 'head'

# Control: other names for the cell, both closures and the mover.
$renamed = Join-Path $Out 'renamed'
foreach ($f in $four) {
    New-Item -ItemType Directory -Force (Split-Path (Join-Path $renamed $f)) | Out-Null
    Copy-Item (Join-Path $tree $f) (Join-Path $renamed $f)
}
Edit (Join-Path $renamed $four[0]) { param($t)
    $t.Replace('let stop = Cell::new(None);', 'let halted = Cell::new(None);').Replace('stop.set(', 'halted.set(').Replace('stop.take()', 'halted.take()').Replace('let mut call = |', 'let mut ask = |').Replace('let mut check = |', 'let mut verify = |').Replace('with_repairs(&mut call, &request, &mut check,', 'drive(&mut ask, &request, &mut verify,') }
Edit (Join-Path $renamed $four[1]) { param($t)
    $t.Replace('pub fn with_repairs(', 'pub fn drive(').Replace('call: &mut dyn FnMut(&CallRequest)', 'first: &mut dyn FnMut(&CallRequest)').Replace('check: &mut dyn FnMut(&str)', 'second: &mut dyn FnMut(&str)').Replace('match call(&asked)', 'match first(&asked)').Replace('= check(&reply.text);', '= second(&reply.text);') }
Find $renamed $four 'renamed'

# Control: the read feeds no condition.
$unguarded = Join-Path $Out 'unguarded'
foreach ($f in $four) {
    New-Item -ItemType Directory -Force (Split-Path (Join-Path $unguarded $f)) | Out-Null
    Copy-Item (Join-Path $tree $f) (Join-Path $unguarded $f)
}
$path = Join-Path $unguarded $four[0]
$lines = [System.Collections.Generic.List[string]](Get-Content $path)
if ($lines[198] -notmatch 'match stop\.take\(\)') { throw 'the read is not where it was' }
$lines[198] = '            let _ = stop.take();'
$lines[199] = '            let verdict = judge(task, &pack_text(text));'
$lines.RemoveRange(200, 5)
Set-Content $path $lines -Encoding utf8NoBOM
Find $unguarded $four 'unguarded'

# Sweep: the project's own Rust sources, vendored crates left out.
$own = '^(native/src|model/src|tools/[^/]+/src)/.*\.rs$'
$old = git -C $repo ls-tree -r --name-only $Before -- native/src model/src tools | Where-Object { $_ -match $own }
$new = git -C $repo ls-files -- native/src model/src tools | Where-Object { $_ -match $own }
Find $tree $old "sweep-$Before"
Find $repo $new 'sweep-head'

# One line per cell that more than one unit touches.
Get-ChildItem $Out -Filter *.json | ForEach-Object {
    $j = Get-Content $_.FullName -Raw | ConvertFrom-Json
    "== $($_.BaseName): files $($j.files), $($j.totals | ConvertTo-Json -Compress)"
    $j.cells | Where-Object verdict -ne 'one unit' | ForEach-Object {
        $n = { param($list) ($list | ForEach-Object { ($_ -split ':')[-1] }) -join ',' }
        "  $($_.cell) [$($_.kind); $($_.verdict)] kept $($_.kept); units $($_.units -join ','); " +
        "written $(& $n $_.written.at); read $(& $n $_.read.at); guards $(& $n $_.guards); " +
        "on a bound name $(& $n $_.guards_on_a_bound_name); " +
        "mover $($_.moved_by.to) at $(& $n $_.moved_by.handed_over_at) calls $(& $n $_.moved_by.calls.at) conditions $(& $n $_.moved_by.conditions)"
    }
    "  calls of a parameter with a namesake: " + (($j.parameter_calls_with_a_namesake | ForEach-Object { "$($_.at) $($_.parameter)" }) -join '; ')
} | Tee-Object (Join-Path $Out 'summary.txt')
