# One live turn of `lesscode-core regenerate run` on a tiny Python module, with the
# endpoint, model and reasoning level of the repository's .env. Shows what the product
# path does with the configured model as it is today.
param([string]$Label = 't1', [string]$ModelEnv = '.env', [string[]]$More = @())
$w = 'D:\BuildArtifacts\lesscode\model-seam'
New-Item -ItemType Directory -Force "$w\frozen\src", "$w\cand-$Label", "$w\turns" | Out-Null
$src = @'
def total_price(items):
    total = 0
    for item in items:
        if item["kind"] == "book":
            total = total + item["price"] * item["count"]
        elif item["kind"] == "food":
            total = total + item["price"] * item["count"]
        else:
            total = total + item["price"] * item["count"]
    return total
'@
[IO.File]::WriteAllText("$w\frozen\src\mod.py", $src.Replace("`r`n", "`n") + "`n")
[IO.File]::WriteAllText("$w\frozen\public-surface.json", '{"modules": {}}')
$req = @{
    scope_root = "$w\frozen"; scope_files = @('src/mod.py'); surface_path = "$w\frozen\public-surface.json"
    definitions = 'total_price(items) returns the sum of price times count over all items; every item has kind, price and count.'
    targets = @{ written_stmts_max = 6 }; contract = 'all functions declared'
    acceptance_command = 'python -m pytest tests -q'; language = 'python'
} | ConvertTo-Json
[IO.File]::WriteAllText("$w\request.json", $req)
# The candidate directory starts as a copy of the frozen tree.
Copy-Item "$w\frozen\*" "$w\cand-$Label" -Recurse -Force
Set-Location 'D:\Source\lesscode'
$exe = Join-Path $env:CARGO_TARGET_DIR 'debug\lesscode-core.exe'
$t0 = Get-Date
$said = & $exe regenerate run --request "$w\request.json" --cand "$w\cand-$Label" --receipt "$w\receipt-$Label.json" `
    --label $Label --frozen "$w\frozen" --turns "$w\turns" --model-env $ModelEnv @More 2>&1 | Out-String
$said.Substring(0, [Math]::Min(3000, $said.Length))
"exit $LASTEXITCODE after $([int]((Get-Date) - $t0).TotalSeconds)s"
Get-ChildItem $w -Recurse -File | ForEach-Object { '{0,8} {1}' -f $_.Length, $_.FullName.Substring($w.Length + 1) }
