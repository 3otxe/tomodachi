# tomodachi — PowerShell integration
# Dot-source this from your $PROFILE:
#   . "C:\path\to\tomodachi.ps1"
#
# Requires: tomodachi-client.exe in PATH

# Guard: don't load if client isn't available
if (-not (Get-Command tomodachi-client.exe -ErrorAction SilentlyContinue)) {
    Write-Warning "tomodachi-client.exe not found in PATH — tomodachi integration disabled"
    return
}

# ── Prompt hook (precmd equivalent) ────────────────────────────────────
# Override the prompt function to report the last exit code to the daemon.

$_tomodachi_original_prompt = $function:prompt

function prompt {
    $exitCode = $LASTEXITCODE
    & tomodachi-client.exe notify --exit $exitCode --cwd (Get-Location).Path --shell powershell | Out-Null
    
    if ($_tomodachi_original_prompt) { & $_tomodachi_original_prompt }
}

# ── PSReadLine Enter handler (preexec equivalent) ─────────────────────
# Captures the command text before it executes.

if (Get-Module -Name PSReadLine -ListAvailable) {
    Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
        $line = $null; $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

        if ($line.Trim().Length -gt 0) {
            & tomodachi-client.exe notify --pending $line --shell powershell | Out-Null
        }

        [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
    }
}

# ── Veto mode ──────────────────────────────────────────────────────────
# Shadow dangerous commands. Uncomment to enable.

# function Remove-Item {
#     tomodachi-client.exe veto Remove-Item @args
# }
