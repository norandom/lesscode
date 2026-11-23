# Prototype gate of the run page: opens pages in a headless browser and reads the marks the page
# puts on itself.
#   ./gate.ps1 -Proto <run-page-proto.exe> -Receipt <plan-run.json> -Plan <plan.json> -Library <file> -Out <new directory>
# The browser is asked over its debugging port, with PowerShell's own web socket client; nothing
# else is measured here. The pages and the replay are made by the Rust program.
param([Parameter(Mandatory)][string]$Proto, [Parameter(Mandatory)][string]$Receipt, [Parameter(Mandatory)][string]$Plan,
      [Parameter(Mandatory)][string]$Library, [Parameter(Mandatory)][string]$Out,
      [string]$Browser = 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe')
$ErrorActionPreference = 'Stop'
if (Test-Path -LiteralPath $Out) { throw "$Out exists" }
if ($Out -notmatch '^D:\\BuildArtifacts\\lesscode\\') { throw 'output must be under D:\BuildArtifacts\lesscode' }
New-Item -ItemType Directory -Force $Out | Out-Null
$profile = Join-Path $Out 'browser-profile'
$port = 9300 + (Get-Random -Maximum 400)

function Open-Page([string]$Address, [string]$Size = '1280,800') {
    Start-Process $Browser -ArgumentList '--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check',
        "--user-data-dir=$profile", "--remote-debugging-port=$port", "--window-size=$Size", $Address -PassThru
}
# What an expression gives in the page, as text.
function Ask([string]$Expression) {
    $target = (Invoke-RestMethod "http://127.0.0.1:$port/json") | Where-Object type -eq 'page' | Select-Object -First 1
    $socket = [System.Net.WebSockets.ClientWebSocket]::new()
    $socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).Wait()
    $ask = @{ id = 1; method = 'Runtime.evaluate'; params = @{ expression = $Expression; returnByValue = $true } } | ConvertTo-Json -Compress -Depth 5
    $socket.SendAsync([ArraySegment[byte]]::new([Text.Encoding]::UTF8.GetBytes($ask)), 'Text', $true, [Threading.CancellationToken]::None).Wait()
    $buffer = [byte[]]::new(1MB); $text = ''
    do { $got = $socket.ReceiveAsync([ArraySegment[byte]]::new($buffer), [Threading.CancellationToken]::None).Result
         $text += [Text.Encoding]::UTF8.GetString($buffer, 0, $got.Count) } until ($got.EndOfMessage)
    $socket.Dispose()
    ($text | ConvertFrom-Json).result.result.value
}
function Marks { Ask 'JSON.stringify(document.documentElement.dataset)' | ConvertFrom-Json }
function Close-Page($Process) { taskkill /PID $Process.Id /T /F 2>$null | Out-Null; Start-Sleep -Milliseconds 400 }
function Address([string]$Path) { [Uri]::new($Path).AbsoluteUri }
$results = [ordered]@{ started = (Get-Date -Format o); proto = (Get-FileHash $Proto).Hash; library = (Get-FileHash $Library).Hash; browser = (Get-Item $Browser).VersionInfo.ProductVersion }

# P4 and P5: fixed situations.
$fixed = Join-Path $Out 'fixtures'
& $Proto fixtures $Receipt $Plan $Library $fixed | Out-Null
$results.fixed = foreach ($name in 'waiting', 'running', 'quiet', 'stopped', 'ended', 'cannot-read', 'damaged', 'sparse', 'large', 'void', 'refused') {
    foreach ($size in '1280,800', '400,800') {
        $p = Open-Page (Address (Join-Path $fixed "$name.html")) $size; Start-Sleep -Milliseconds 1800
        $m = Marks; Close-Page $p
        [ordered]@{ page = $name; asked = $size; width = $m.width; state = $m.state; failures = $m.failures; sideways = $m.sideways }
    }
}

# P1 and P2: one page, opened from disk, while the data beside it is rewritten.
$live = Join-Path $Out 'live'
New-Item -ItemType Directory -Force $live | Out-Null
$replay = Start-Process $Proto -ArgumentList 'replay', $Receipt, $Plan, $Library, $live, '--tick-ms', '30', '--read-ms', '50' -NoNewWindow -PassThru `
    -RedirectStandardOutput (Join-Path $live 'replay.out') -RedirectStandardError (Join-Path $live 'replay.err')
Start-Sleep -Milliseconds 200
$p = Open-Page (Address (Join-Path $live 'run.html'))
$seen = for ($i = 0; $i -lt 14; $i++) { Start-Sleep -Milliseconds 1000; $m = Marks; "$($m.state) reads $($m.reads) different $($m.different) misses $($m.misses) failures $($m.failures)"; if ($m.state -eq 'ended') { break } }
$last = Marks; Close-Page $p
$replay.WaitForExit(30000) | Out-Null
$results.following = [ordered]@{ each_second = @($seen); last = $last; writer = (Get-Content (Join-Path $live 'replay.out') -Raw | ConvertFrom-Json) }

# P3: the report alone.
$alone = Join-Path $Out 'alone'
New-Item -ItemType Directory -Force $alone | Out-Null
Copy-Item (Join-Path $live 'run.html') (Join-Path $alone 'report.html')
$p = Open-Page (Address (Join-Path $alone 'report.html')); Start-Sleep -Milliseconds 1800
$m = Marks; $counts = Ask 'document.querySelector(".facts") ? document.querySelector(".facts").innerText.replace(/\s+/g, " ") : ""'; Close-Page $p
$report = Get-Content (Join-Path $alone 'report.html') -Raw
$results.alone = [ordered]@{ state = $m.state; failures = $m.failures; counts = $counts; bytes = (Get-Item (Join-Path $alone 'report.html')).Length
    addresses = @([regex]::Matches($report, 'https?://[^\s"''<>)]+') | ForEach-Object Value | Sort-Object -Unique) }

# P6: subscribed to the tool.
$served = Join-Path $Out 'served'
New-Item -ItemType Directory -Force $served | Out-Null
$replay = Start-Process $Proto -ArgumentList 'replay', $Receipt, $Plan, $Library, $served, '--tick-ms', '30', '--serve', '--linger-ms', '4000' -NoNewWindow -PassThru `
    -RedirectStandardOutput (Join-Path $served 'replay.out') -RedirectStandardError (Join-Path $served 'replay.err')
for ($i = 0; $i -lt 50 -and -not (Test-Path (Join-Path $served 'address.txt')); $i++) { Start-Sleep -Milliseconds 100 }
$address = Get-Content (Join-Path $served 'address.txt') -Raw
$base = ($address -split '/')[0..2] -join '/'
$without = try { (Invoke-WebRequest "$base/" -SkipHttpErrorCheck -TimeoutSec 5).StatusCode } catch { "no answer: $($_.Exception.Message)" }
$wrong = try { (Invoke-WebRequest "$base/not-the-secret/events" -SkipHttpErrorCheck -TimeoutSec 5).StatusCode } catch { "no answer: $($_.Exception.Message)" }
$p = Open-Page $address
$seen = for ($i = 0; $i -lt 14; $i++) { Start-Sleep -Milliseconds 1000; $m = Marks; "$($m.state) reads $($m.reads) different $($m.different) failures $($m.failures)"; if ($m.state -eq 'ended') { break } }
$last = Marks; Close-Page $p
$replay.WaitForExit(30000) | Out-Null
$results.subscribed = [ordered]@{ each_second = @($seen); last = $last; without_the_secret = $without; with_a_wrong_secret = $wrong }

# P6, second part: the tool goes away before the run has ended.
$gone = Join-Path $Out 'gone'
New-Item -ItemType Directory -Force $gone | Out-Null
$replay = Start-Process $Proto -ArgumentList 'replay', $Receipt, $Plan, $Library, $gone, '--tick-ms', '10', '--serve', '--stop-early' -NoNewWindow -PassThru `
    -RedirectStandardOutput (Join-Path $gone 'replay.out') -RedirectStandardError (Join-Path $gone 'replay.err')
for ($i = 0; $i -lt 50 -and -not (Test-Path (Join-Path $gone 'address.txt')); $i++) { Start-Sleep -Milliseconds 100 }
$p = Open-Page (Get-Content (Join-Path $gone 'address.txt') -Raw)
$replay.WaitForExit(30000) | Out-Null; Start-Sleep -Milliseconds 2500
$m = Marks; Close-Page $p
$results.gone = [ordered]@{ state = $m.state; different = $m.different; failures = $m.failures }

# P7: pictures.
$pictures = Join-Path $Out 'pictures'
New-Item -ItemType Directory -Force $pictures | Out-Null
foreach ($shot in @('running', ''), @('ended', ''), @('ended', '--force-dark-mode'), @('stopped', ''), @('refused', ''), @('void', '--force-dark-mode')) {
    $name, $extra = $shot
    $png = Join-Path $pictures ("$name" + $(if ($extra) { '-dark' }) + '.png')
    $arguments = @('--headless=new', '--disable-gpu', '--no-first-run', "--user-data-dir=$profile", '--window-size=1280,800', '--virtual-time-budget=3000', "--screenshot=$png")
    if ($extra) { $arguments += $extra }
    & $Browser @arguments (Address (Join-Path $fixed "$name.html")) 2>$null | Out-Null
}
$results.pictures = @(Get-ChildItem $pictures | ForEach-Object Name)
$results.ended = (Get-Date -Format o)
$results | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Out 'gate.json') -Encoding utf8NoBOM
