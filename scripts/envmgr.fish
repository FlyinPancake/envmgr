#!/usr/bin/env fish

# envmgr shell integration for Fish
# Add this to your ~/.config/fish/config.fish or source it manually

function __envmgr_apply_output --argument-names output
    for line in $output
        if string match -rq '^set -gx ' -- $line
            eval $line
        else if string match -rq '^export ' -- $line
            set -l kv (string replace -r '^export ' '' -- $line)
            set -l key (string split -m1 '=' -- $kv)[1]
            set -l val (string split -m1 '=' -- $kv)[2]
            eval "set -gx $key $val"
        end
    end
end

function envmgr
    if test (count $argv) -ge 1 -a "$argv[1]" = "use"
        set -l output (command envmgr $argv 2>&1)
        set -l exit_code $status
        if test $exit_code -eq 0
            __envmgr_apply_output "$output"
            if test (count $argv) -ge 2
                echo "Switched to environment: $argv[2]"
            end
        else
            echo $output
            return $exit_code
        end
    else
        command envmgr $argv
    end
end

function envmgr_use
    if test (count $argv) -ne 1
        echo "Usage: envmgr_use <environment>"
        return 1
    end
    
    set env_name $argv[1]
    
    # Run envmgr use and capture the output
    set -l output (envmgr use $env_name 2>&1)
    set -l exit_code $status
    
    if test $exit_code -eq 0
        # Parse and execute lines for both fish and sh styles
        for line in $output
            if string match -rq '^set -gx ' -- $line
                eval $line
            else if string match -rq '^export ' -- $line
                set -l kv (string replace -r '^export ' '' -- $line)
                set -l key (string split -m1 '=' -- $kv)[1]
                set -l val (string split -m1 '=' -- $kv)[2]
                eval "set -gx $key $val"
            end
        end
        echo "Switched to environment: $env_name"
    else
        echo $output
        return $exit_code
    end
end

# Optional: Create an alias for easier usage
alias eu="envmgr_use"

# Optional: Show current environment in prompt
function envmgr_prompt
    if set -q ENVMGR_CURRENT_ENV
        echo "($ENVMGR_CURRENT_ENV) "
    end
end

# Optional: Auto-complete environments
complete -c envmgr_use -a "(command envmgr list 2>/dev/null | grep -E '^  [a-zA-Z]' | sed 's/^  //' | sed 's/ .*//')"
