# Runs the prototype's one turn against an endpoint file, optionally with another model
# name. The prompts are the ones `regenerate run` wrote for the probe task of R102.
param([Parameter(Mandatory)][string]$EnvFile, [string]$Model = '', [string]$Level = '')
$turn = 'D:\BuildArtifacts\lesscode\model-seam\turns\t2'
$exe = 'D:\BuildArtifacts\cargo-target\model-seam-prototype\debug\model-seam-prototype.exe'
if ($Model) { $env:LESSCODE_MODEL_NAME = $Model }
if ($Level) { $env:LESSCODE_MODEL_REASONING_EFFORT = $Level }
& $exe $EnvFile "$turn\system.txt" "$turn\user.txt"
