# The repair loop of model/src/repair.rs as an SMT-LIB term over r1, r2, r3.
# A reply code is 0 complete and admissible, 1 complete and rejected, 2 cut
# off, 3 refused, 4 failed call. Two repairs are allowed. The result is ten
# times the number of the last call, plus 0 accepted, 1 rejected, 2 failed.
# $Pass gives, for a symbol, the term that says the reply is accepted: a
# failed call ends the loop, a pass is accepted, a refusal or the last call
# is rejected, anything else is asked again.
function Loop([scriptblock]$Pass) {
    $rest = '31'
    foreach ($i in 3, 2, 1) {
        $r = "r$i"
        $ends = if ($i -eq 3) { 'true' } else { "(= $r 3)" }
        $rest = "(ite (= $r 4) $($i * 10 + 2) (ite $(& $Pass $r) $($i * 10) (ite $ends $($i * 10 + 1) $rest)))"
    }
    $rest
}

# One arm: a frozen test binary and the harness test to run in it.
function Arm([string]$Label, [string]$Exe, [string]$Test, [string]$Expression) {
    [ordered]@{ label = $Label; expression = $Expression
        command = @('pwsh', '-NoProfile', '-File', "$PSScriptRoot\run.ps1", $Exe, $Test, '{inputs}', '{output}') }
}

function Sha([string]$Path) { (Get-FileHash $Path -Algorithm SHA256).Hash.ToLower() }

# The target file for the given arms and pairs over all 125 histories.
function Target([string]$Z3, [string[]]$Exes, [object[]]$Arms, [object[]]$Pairs) {
    [ordered]@{
        solver  = [ordered]@{ path = $Z3; sha256 = (Sha $Z3) }
        pins    = @($Exes | ForEach-Object { [ordered]@{ path = $_; sha256 = (Sha $_) } })
        symbols = @('r1', 'r2', 'r3' | ForEach-Object { [ordered]@{ name = $_; min = 0; max = 4 } })
        sort    = 'Int'
        arms    = $Arms
        compare = $Pairs
        case    = [ordered]@{ id = '{id}'; replies = @('{r1}', '{r2}', '{r3}') }
    }
}
