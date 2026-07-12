# Define aliases.
alias ls='eza -a'
# Reuse ls completions for eza (avoids defining a separate completion function)
compdef eza=ls
alias -- -='cd -'  # -- prevents - being parsed as a flag; cd - jumps to previous directory
alias c=' clear'
alias lg='lazygit -ucd="$XDG_CONFIG_HOME/lazygit"'
alias n='nvim'
alias e=' exit'
alias ip='ipconfig getifaddr en0'
alias pp="pnpm"

function y {
	local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
	yazi "$@" --cwd-file="$tmp"
	if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
		builtin cd -- "$cwd"
	fi
	rm -f -- "$tmp"
}
