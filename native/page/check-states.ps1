<#
.SYNOPSIS
Opens the run page in a headless browser for fixed situations and checks the
state the page gives itself (.kiro/specs/run-dashboard, task 4.2).

.DESCRIPTION
For each situation a page is written from run.html and the library beside
this script, with the situation's data inside, as the tool writes a report.
The page is opened from disk, and the marks it puts on itself are read over
the browser's debugging port: its state, failures of its code, and whether
it scrolls sideways. Some situations are also asked for what they show.
Where a window does not go as narrow as a size asks, the page is given that
width over the same port, and the record says so. A picture of every page
is kept.

This is not one of the unit tests. It needs PowerShell 7 and a browser of
the Chromium family (Edge, Chrome, Chromium). It writes states.json into
-Out and ends with 1 when a situation is not as expected.

.EXAMPLE
./native/page/check-states.ps1 -Out D:\BuildArtifacts\lesscode\run-dashboard\states-1
#>
param(
    [Parameter(Mandatory)][string]$Out,
    [string]$Browser,
    [string[]]$Sizes = @('1280,800', '400,800')
)
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if (-not $Browser) {
    $known = @(
        "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe",
        "$env:ProgramFiles\Google\Chrome\Application\chrome.exe",
        '/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge', '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    ) | Where-Object { $_ -and (Test-Path -LiteralPath $_) }
    $named = 'microsoft-edge', 'google-chrome', 'chromium', 'chromium-browser' |
        ForEach-Object { (Get-Command $_ -ErrorAction SilentlyContinue).Source } | Where-Object { $_ }
    $Browser = @($known) + @($named) | Select-Object -First 1
    if (-not $Browser) { throw 'no browser found; name one with -Browser' }
}
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path -LiteralPath $Out).Path
$pages = Join-Path $Out 'pages'
New-Item -ItemType Directory -Force $pages | Out-Null
$pictures = Join-Path $Out 'pictures'
New-Item -ItemType Directory -Force $pictures | Out-Null
$browserProfile = Join-Path $Out 'browser-profile'
$port = 9300 + (Get-Random -Maximum 400)

# The page as the tool builds it: the library in its place, then one marker filled.
function Split-At([string]$Text, [string]$Marker) {
    $at = $Text.IndexOf($Marker, [StringComparison]::Ordinal)
    if ($at -lt 0) { throw "the page has no $Marker" }
    $Text.Substring(0, $at), $Text.Substring($at + $Marker.Length)
}
$before, $after = Split-At ([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'run.html'))) '/*LIB*/'
$template = $before + [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'htm-preact-standalone.umd.js')) + $after
function Write-Page([string]$Name, $Data, [string]$Events) {
    $page = $template
    if ($null -ne $Data) {
        $before, $after = Split-At $page '/*DATA*/null'
        $page = $before + ($Data | ConvertTo-Json -Depth 12 -Compress).Replace('<', '\u003c') + $after
    }
    if ($Events) {
        $before, $after = Split-At $page '/*EVENTS*/null'
        $page = $before + ($Events | ConvertTo-Json -Compress) + $after
    }
    $path = Join-Path $pages "$Name.html"
    [IO.File]::WriteAllText($path, $page, [Text.UTF8Encoding]::new($false))
    [Uri]::new($path).AbsoluteUri
}

function Open-Page([string]$Address, [string]$Size) {
    Start-Process $Browser -PassThru -ArgumentList '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
        "--user-data-dir=$browserProfile", "--remote-debugging-port=$port", "--window-size=$Size", $Address
}
function Close-Page($Process) {
    if ($script:socket) { $script:socket.Dispose(); $script:socket = $null }
    try { $Process.Kill($true) } catch { }
    $Process.WaitForExit(5000) | Out-Null
    Start-Sleep -Milliseconds 300
}
# One connection to the page for as long as it is open: a width that is set
# over the connection lasts only as long as the connection does.
$script:socket = $null
$script:sent = 0
function Connect-Page {
    $target = $null
    for ($i = 0; $i -lt 80 -and -not $target; $i++) {
        try { $target = (Invoke-RestMethod "http://127.0.0.1:$port/json" -TimeoutSec 2) | Where-Object type -eq 'page' | Select-Object -First 1 } catch { }
        if (-not $target) { Start-Sleep -Milliseconds 100 }
    }
    if (-not $target) { throw 'the browser shows no page' }
    $script:socket = [Net.WebSockets.ClientWebSocket]::new()
    $script:socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait()
}
# Asks the browser for one thing and gives its answer.
function Send-Browser([string]$Method, [hashtable]$With = @{}) {
    $script:sent++
    $id = $script:sent
    $ask = @{ id = $id; method = $Method; params = $With } | ConvertTo-Json -Compress -Depth 6
    $limit = [Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds(30))
    try {
        $script:socket.SendAsync([ArraySegment[byte]]::new([Text.Encoding]::UTF8.GetBytes($ask)), 'Text', $true, $limit.Token).Wait()
        $buffer = [byte[]]::new(1MB)
        while ($true) {
            $bytes = [IO.MemoryStream]::new()
            do {
                $got = $script:socket.ReceiveAsync([ArraySegment[byte]]::new($buffer), $limit.Token).Result
                $bytes.Write($buffer, 0, $got.Count)
            } until ($got.EndOfMessage)
            $answer = [Text.Encoding]::UTF8.GetString($bytes.ToArray()) | ConvertFrom-Json
            if ($answer.id -eq $id) {
                if ($answer.error) { throw "$Method`: $($answer.error.message)" }
                return $answer.result
            }
        }
    } finally { $limit.Dispose() }
}
# What an expression gives in the page.
function Ask([string]$Expression) {
    (Send-Browser 'Runtime.evaluate' @{ expression = $Expression; returnByValue = $true }).result.value
}
# The width the page has. Where the browser's window does not go as narrow as
# asked, the page is given the width over the debugging port, as the
# browser's own tools for narrow screens do.
function Set-Width([string]$Size) {
    $width, $height = $Size -split ',' | ForEach-Object { [int]$_ }
    $given = [int](Ask 'window.innerWidth')
    if ($given -le $width) { return [ordered]@{ window = $given; set = $false } }
    [void](Send-Browser 'Emulation.setDeviceMetricsOverride' @{ width = $width; height = $height; deviceScaleFactor = 1; mobile = $false })
    # The page notes its width once a second.
    Start-Sleep -Milliseconds 1300
    [ordered]@{ window = $given; set = $true }
}
function Save-Picture([string]$Name) {
    $picture = Send-Browser 'Page.captureScreenshot' @{ format = 'png' }
    [IO.File]::WriteAllBytes((Join-Path $pictures "$Name.png"), [Convert]::FromBase64String($picture.data))
}
# The marks of the page, once it has given itself a state.
function Marks {
    # A page that is still loading may have no root element yet.
    $ask = 'document.documentElement ? JSON.stringify(document.documentElement.dataset) : "{}"'
    for ($i = 0; $i -lt 60; $i++) {
        $marks = "$(Ask $ask)" | ConvertFrom-Json
        if ($marks.state) { Start-Sleep -Milliseconds 600; return ("$(Ask $ask)" | ConvertFrom-Json) }
        Start-Sleep -Milliseconds 150
    }
    throw 'the page gave itself no state'
}
function Shown { Ask 'document.body ? document.body.innerText : ""' }
function Stamp([int]$SecondsAgo) { [DateTime]::UtcNow.AddSeconds(-$SecondsAgo).ToString('yyyy-MM-ddTHH:mm:ss.fffZ') }

# A run of `step plan`, midway and ended, in the shape the tool writes.
function Stage($Id, $Title, $Status, $Done, $Total, $Seconds) {
    [ordered]@{ id = $Id; title = $Title; what = "What the stage $Id does."; status = $Status; done = $Done; total = $Total
        seconds = $(if ($Status -eq 'ended') { $Seconds }); seconds_so_far = $(if ($Status -eq 'going on') { $Seconds })
        seconds_left = $(if ($Status -eq 'going on' -and $Done -ge 5) { 40 }); reason = $null }
}
function Tally($File, $Function, $Noticed, $Unnoticed, $Left) {
    [ordered]@{ file = $File; function = $Function; noticed = $Noticed; unnoticed = $Unnoticed; not_built = 0; timed_out = 0; not_judged = 0
        left = $Left; unprotected = ($Noticed -eq 0 -and $Unnoticed -gt 0 -and $Left -eq 0); reaches_out = @() }
}
function Run([bool]$Ended, [int]$HeardSecondsAgo) {
    $left = if ($Ended) { 0 } else { 4 }
    [ordered]@{
        schema = 1; sequence = 14; tool = @{ name = 'lesscode-core'; version = '2.1.0' }; command = 'step plan'
        started = Stamp ($HeardSecondsAgo + 90); heartbeat = Stamp $HeardSecondsAgo; ended = $Ended
        outcome = $(if ($Ended) { 'The plan is written' }); outcome_detail = $(if ($Ended) { '1 of 3 functions with planted faults are unprotected.' })
        exit_code = $(if ($Ended) { 0 }); seconds = 90.5
        subject = [ordered]@{ repository = 'D:/Source/example'; branch = 'main'; commit = '0123456789abcdef0123456789abcdef01234567'; dirty = $false; on_main_branch = $true }
        copies = [ordered]@{ directory = 'D:/BuildArtifacts/example/work'; planned = 2; working = 2; dropped = @(); bytes = $null }
        commands = @(, @('cargo', 'test')); writes_in_repository = @()
        tests = [ordered]@{ command_given = $true; passed_unchanged = $true; tests_run = 12; reason = $null }
        stages = @(
            (Stage 'prepare' 'Copy the project and run your tests on the unchanged code' 'ended' $null $null 30),
            (Stage 'faults' 'Plant faults' $(if ($Ended) { 'ended' } else { 'going on' }) (12 - $left) 12 55),
            (Stage 'close' 'Check the copies and write the plan' $(if ($Ended) { 'ended' } else { 'not begun' }) $null $null 5))
        faults = [ordered]@{
            total = 12; done = 12 - $left; noticed = 5; unnoticed = 7 - $left; not_built = 0; timed_out = 0; not_judged = 0; not_compiled_here = $null
            example = [ordered]@{ file = 'src/app.rs'; line = 2; what_was_changed = 'negate'; text = 'a < b'; outcome = 'noticed: a test failed' }
            files = @((Tally 'src/app.rs' $null 5 3 0), (Tally 'src/draw.rs' $null 0 (4 - $left) $left))
            functions = @((Tally 'src/app.rs' 'step' 5 1 0), (Tally 'src/app.rs' 'parse' 0 2 0), (Tally 'src/draw.rs' 'paint' 0 (4 - $left) $left))
        }
        next = @('Read the tab Unprotected.', 'Write tests for a function before you change it.')
    }
}
function Large {
    $run = Run $true 5
    $run.faults.files = @(0..19 | ForEach-Object { Tally ('src/renderer/part_{0:d2}.rs' -f $_) $null 200 100 0 })
    $run.faults.functions = @(0..1999 | ForEach-Object {
            $name = ('a_function_with_a_very_long_name' * 4 + "_$_").Substring(0, 120)
            $one = Tally ('src/renderer/part_{0:d2}.rs' -f ($_ % 20)) $name ($_ % 5) ($_ % 3) 0
            $one.reaches_out = @('create_surface', 'write_texture'); $one })
    $run
}
function Void {
    $run = Run $true 5
    $run.tests = [ordered]@{ command_given = $false; passed_unchanged = $null; tests_run = $null; reason = $null }
    $run.outcome = 'Could not judge'; $run.outcome_detail = 'No test command was given, so nothing below was measured.'; $run.exit_code = 2
    $run.stages = @($run.stages | ForEach-Object { $_.status = 'not begun'; $_.seconds = $null; $_.done = $null; $_ })
    $run.faults = [ordered]@{ total = 0; done = 0; noticed = 0; unnoticed = 0; not_built = 0; files = @(); functions = @() }
    $run
}
function Refused {
    $run = Run $true 5
    $run.command = 'step check'; $run.outcome = 'Refused'; $run.outcome_detail = 'The step is not accepted as it stands.'; $run.exit_code = 1
    $run.report = [ordered]@{
        lenses = @(
            [ordered]@{ lens = 'Size'; measure = 'tokens of the file'; before = 4495; after = 4567; expected = 'may rise by 60'; moved = $false; measured = $true },
            [ordered]@{ lens = 'Protection'; measure = 'planted faults unnoticed'; before = ''; after = ''; expected = 'does not rise'; measured = $false })
        refusal = @{ reasons = @('the scope grows from 4495 to 4567 tokens; the declaration allows 60 more') }
    }
    $run
}
# Every field of another kind than the page expects.
function Wrong {
    [ordered]@{ schema = 1; heartbeat = 7; ended = $true; tool = 'a text'; command = @(1, 2); seconds = 'long'; exit_code = 'none'
        subject = 'a text'; copies = [ordered]@{ dropped = @($null, 5, 'x'); planned = 'two' }; commands = @(5, $null, 'x'); writes_in_repository = 3
        tests = @(); stages = @($null, 5, 'x', @{}, @{ id = 'faults'; status = 7; done = 'x' })
        faults = [ordered]@{ total = 'many'; files = @($null, @{}, @{ file = 9 }); functions = 'none'; example = 7 }
        report = [ordered]@{ lenses = @($null, 3, @{}); refusal = 'yes' }; next = 'x'; outcome = 5; outcome_detail = @{} }
}

# Name, the state expected, the data inside, the address of a stream, and what the page must and must not show.
$situations = @(
    @{ name = 'waiting'; state = 'waiting'; data = { $null }; shows = @('Waiting for lesscode') }
    @{ name = 'waiting-no-answer'; state = 'waiting'; data = { $null }; events = 'http://127.0.0.1:9/none/events'; shows = @('The tool does not answer') }
    @{ name = 'running'; state = 'running'; data = { Run $false 2 }; shows = @('Running for', '8 of 12 done') }
    @{ name = 'quiet'; state = 'quiet'; data = { Run $false 30 }; shows = @('No word from the tool for') }
    @{ name = 'stopped'; state = 'stopped'; data = { Run $false 120 }; shows = @('The tool stopped without finishing', 'Stopped after', 'was going on when the tool stopped'); hides = @('Running for', '▶') }
    @{ name = 'ended'; state = 'ended'; data = { Run $true 5 }; shows = @('The plan is written', 'Took 1 min 31 s'); hides = @('Running for', '▶'); folds = $true }
    @{ name = 'cannot-read'; state = 'cannot-read'; data = { @{ schema = 99 } }; shows = @('cannot read', 'version 99', 'reads version 1') }
    @{ name = 'damaged'; state = 'cannot-read'; data = { 'this is not a run' }; shows = @('cannot read', 'found a text') }
    @{ name = 'sparse'; state = 'running'; data = { @{ schema = 1; heartbeat = Stamp 1 } }; shows = @('not known') }
    @{ name = 'wrong'; state = 'ended'; data = { Wrong } }
    @{ name = 'large'; state = 'ended'; data = { Large }; folds = $true }
    @{ name = 'void'; state = 'ended'; data = { Void }; shows = @('Could not judge', 'No test command was given'); hides = @('✓') }
    @{ name = 'refused'; state = 'ended'; data = { Refused }; shows = @('Refused', 'Fix the step') }
)

$version = if ($IsWindows) { (Get-Item $Browser).VersionInfo.ProductVersion } else { (& $Browser --version) -join ' ' }
$results = [ordered]@{ started = (Get-Date -Format o); machine = [Environment]::MachineName; system = [Runtime.InteropServices.RuntimeInformation]::OSDescription
    browser = $Browser; browser_version = $version; page_sha256 = (Get-FileHash (Join-Path $PSScriptRoot 'run.html')).Hash.ToLowerInvariant() }
try {
$rows = foreach ($situation in $situations) {
    foreach ($size in $Sizes) {
        # Written just before it is opened: the states depend on how old the last word is.
        $address = Write-Page $situation.name (& $situation.data) $situation.events
        $process = Open-Page $address $size
        try {
            Connect-Page
            [void](Marks); $width = Set-Width $size
            $marks = Marks; $shown = Shown
            # What comes a moment later, such as a stream that is refused, is waited for.
            for ($i = 0; $i -lt 8 -and @($situation.shows | Where-Object { $_ -and -not "$shown".Contains($_) }); $i++) {
                Start-Sleep -Milliseconds 500; $marks = Marks; $shown = Shown
            }
            $wrong = [Collections.Generic.List[string]]::new()
            if ($marks.state -ne $situation.state) { $wrong.Add("state $($marks.state), expected $($situation.state)") }
            if ($marks.failures -ne '0') { $wrong.Add("failures $($marks.failures): $($marks.failure)") }
            if ($marks.sideways -ne 'false') { $wrong.Add('the page scrolls sideways') }
            if (-not "$shown".Trim()) { $wrong.Add('the page is blank') }
            if ([int]$marks.width -gt [int]($size -split ',')[0]) { $wrong.Add("the page is $($marks.width) wide, asked for $size") }
            Save-Picture "$($situation.name)-$($marks.width)"
            foreach ($text in @($situation.shows)) { if ($text -and -not "$shown".Contains($text)) { $wrong.Add("does not show: $text") } }
            foreach ($text in @($situation.hides)) { if ($text -and "$shown".Contains($text)) { $wrong.Add("shows: $text") } }
            $fold = $null
            if ($situation.folds) {
                # Folding one file: its functions go, the other files keep theirs.
                $rowsBefore = [int](Ask 'document.querySelectorAll(".d2").length')
                [void](Ask 'document.querySelector("button.d1").click()'); Start-Sleep -Milliseconds 400
                $rowsAfter = [int](Ask 'document.querySelectorAll(".d2").length')
                $expanded = Ask 'Array.from(document.querySelectorAll("button.d1")).map(function (b) { return b.getAttribute("aria-expanded"); }).join(",")'
                $fold = "$rowsBefore to $rowsAfter rows; $($expanded.Substring(0, [Math]::Min(22, $expanded.Length)))"
                if (-not ($rowsAfter -gt 0 -and $rowsAfter -lt $rowsBefore) -or -not $expanded.StartsWith('false,true')) { $wrong.Add("folding one file: $fold") }
                $marks = Marks
                if ($marks.failures -ne '0') { $wrong.Add("failures after folding: $($marks.failure)") }
            }
            [ordered]@{ page = $situation.name; asked = $size; width = $marks.width; window_width = $width.window; width_set_over_the_port = $width.set
                state = $marks.state; failures = $marks.failures; sideways = $marks.sideways; fold = $fold; wrong = @($wrong) }
        } finally { Close-Page $process }
    }
}

# Drawing that fails must give the state "cannot read", not a blank page.
$address = Write-Page 'drawing-fails' (Run $true 5) $null
$process = Open-Page $address $Sizes[0]
try {
    Connect-Page
    [void](Marks)
    [void](Ask 'Date.prototype.toLocaleTimeString = function () { throw new Error("made to fail by the check"); }; 1')
    Start-Sleep -Milliseconds 2500
    $marks = Marks; $shown = Shown
    $wrong = [Collections.Generic.List[string]]::new()
    if ($marks.state -ne 'cannot-read') { $wrong.Add("state $($marks.state), expected cannot-read") }
    if ([int]$marks.failures -lt 1) { $wrong.Add('the failure was not counted') }
    foreach ($text in 'cannot read', 'Drawing the page failed', 'made to fail by the check') { if (-not "$shown".Contains($text)) { $wrong.Add("does not show: $text") } }
    $rows += [ordered]@{ page = 'drawing-fails'; asked = $Sizes[0]; width = $marks.width; window_width = [int]$marks.width; width_set_over_the_port = $false
        state = $marks.state; failures = $marks.failures; sideways = $marks.sideways; fold = $null; wrong = @($wrong) }
} finally { Close-Page $process }
} finally {
    # The browser's own files are of no use after the run, also when it broke off.
    Remove-Item -LiteralPath $browserProfile -Recurse -Force -ErrorAction SilentlyContinue
}

$results.rows = @($rows)
$results.ended = (Get-Date -Format o)
$bad = @($rows | Where-Object { $_.wrong.Count })
$results.passed = -not $bad
$results | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Out 'states.json') -Encoding utf8NoBOM
$rows | ForEach-Object { '{0,-18} {1,-9} width {2,-5} {3,-12} failures {4} {5}' -f $_.page, $_.asked, "$($_.width)$(if ($_.width_set_over_the_port) { '*' })", $_.state, $_.failures, ($_.wrong -join '; ') }
if ($rows | Where-Object width_set_over_the_port) { '* the window was wider; the width was set over the debugging port' }
"$($rows.Count) pages opened in $version; $($bad.Count) not as expected; record: $(Join-Path $Out 'states.json')"
exit $(if ($bad) { 1 } else { 0 })
