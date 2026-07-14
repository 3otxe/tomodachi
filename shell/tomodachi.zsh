# tomodachi — zsh integration
# Source this file from your ~/.zshrc:
#   source /path/to/tomodachi.zsh
#
# Requires: tomodachi-client.exe in PATH

# Guard: don't load if client isn't available
if ! command -v tomodachi-client.exe &>/dev/null; then
    return 0
fi

autoload -Uz add-zsh-hook

# precmd: fires after every command finishes, before the prompt is drawn.
# Reports the exit code and current directory to the daemon.
_tomodachi_precmd() {
    local last_exit=$?
    tomodachi-client.exe notify --exit "$last_exit" --cwd "$PWD" --shell zsh &
}

# preexec: fires just before a command executes.
# Reports the pending command text to the daemon.
_tomodachi_preexec() {
    tomodachi-client.exe notify --pending "$1" --shell zsh &
}

add-zsh-hook precmd  _tomodachi_precmd
add-zsh-hook preexec _tomodachi_preexec

# ── Veto mode ──────────────────────────────────────────────────────────
# Shadow dangerous commands with wrapper functions.
# These check with the daemon before executing.
# Use --yolo to bypass.

_tomodachi_veto_rm() {
    tomodachi-client.exe veto rm "$@"
}

# Uncomment to enable veto mode for rm:
# alias rm='_tomodachi_veto_rm'
