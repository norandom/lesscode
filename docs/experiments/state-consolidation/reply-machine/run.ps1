# Run one harness test of a frozen guide-optimizer test binary on the
# histories bound-check wrote. Usage: run.ps1 EXE TEST INPUTS OUTPUT
param([string]$Exe, [string]$Test, [string]$Inputs, [string]$Output)
$env:HISTORY_IN = $Inputs
$env:HISTORY_OUT = $Output
& $Exe --exact "adapter::trials::$Test" --ignored --test-threads 1
exit $LASTEXITCODE
