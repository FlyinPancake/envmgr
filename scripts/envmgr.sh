#!/bin/bash

# envmgr shell integration for Bash/Zsh
# Add this to your ~/.bashrc or ~/.zshrc or source it manually

# Wrapper that applies `envmgr use` to the current shell
function envmgr() {
    if [ "$1" = "use" ]; then
        local output
        output=$(command envmgr "$@" 2>&1)
        local exit_code=$?
        if [ $exit_code -eq 0 ]; then
            while IFS= read -r line; do
                if [[ $line == export* ]]; then
                    eval "$line"
                fi
            done <<< "$output"
            if [ -n "$2" ]; then
                echo "Switched to environment: $2"
            fi
        else
            echo "$output"
            return $exit_code
        fi
    else
        command envmgr "$@"
    fi
}

function envmgr_use() {
    if [ $# -ne 1 ]; then
        echo "Usage: envmgr_use <environment>"
        return 1
    fi
    envmgr use "$1"
}

# Optional: Create an alias for easier usage
alias eu="envmgr_use"

# Optional: Show current environment in prompt
function envmgr_prompt() {
    if [ -n "$ENVMGR_CURRENT_ENV" ]; then
        echo "($ENVMGR_CURRENT_ENV) "
    fi
}

# Optional: Auto-complete environments (Bash)
if [ -n "$BASH_VERSION" ]; then
    _envmgr_use_complete() {
        local cur="${COMP_WORDS[COMP_CWORD]}"
        COMPREPLY=($(compgen -W "$(command envmgr list 2>/dev/null | grep -E '^  [a-zA-Z]' | sed 's/^  //' | sed 's/ .*//')" -- "$cur"))
    }
    complete -F _envmgr_use_complete envmgr_use
    complete -F _envmgr_use_complete eu
fi
