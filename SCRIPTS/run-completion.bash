#!/bin/bash
# Bash completion for run script
# Source this in ~/.bashrc

_run_completion() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local projects="toofoo helios mcad ecad simulations blog learn website list help"
    COMPREPLY=($(compgen -W "$projects" -- "$cur"))
}

complete -F _run_completion run
