# Gate P1: one prototype executable against the six first cases of requirement 2.7.
# Endpoint files live outside the repository, except the repository's own .env.
param([Parameter(Mandatory)][string]$Exe, [int[]]$Only = @(1, 2, 3, 4, 5, 6))
$turn = 'D:\BuildArtifacts\lesscode\model-seam\turns\t2'
$home_env = Join-Path $env:USERPROFILE '.lesscode'
$cases = @(
    @{ id = 1; name = 'GLM 5.3, Mistral API'; file = "$home_env\mistral.env"; model = 'zai-glm-5-3' },
    @{ id = 2; name = 'Mistral Large, Mistral API'; file = "$home_env\mistral.env"; model = '' },
    @{ id = 3; name = 'gpt-6.1-sol, OpenAI API'; file = "$home_env\openai.env"; model = '' },
    @{ id = 4; name = 'Anthropic'; file = "$home_env\model.env"; model = '' },
    @{ id = 5; name = 'GLM 5.3, local endpoint'; file = 'D:\Source\lesscode\.env'; model = '' },
    @{ id = 6; name = 'Mistral Large, local endpoint'; file = 'D:\Source\lesscode\.env'; model = 'mistral-large-latest' }
)
foreach ($case in $cases | Where-Object { $_.id -in $Only }) {
    "--- $($case.id) $($case.name)"
    if ($case.model) { $env:LESSCODE_MODEL_NAME = $case.model }
    else { Remove-Item Env:LESSCODE_MODEL_NAME -ErrorAction SilentlyContinue }
    $said = (& $Exe $case.file "$turn\system.txt" "$turn\user.txt" 2>&1 | Out-String).Trim()
    $said.Substring(0, [Math]::Min(430, $said.Length))
}
Get-Date -Format 'HH:mm'
