# Define aliases.
alias ls='eza -a'
alias c='clear'
alias lg='lazygit -ucd="$HOME/.config/lazygit"'
alias n='nvim'
alias e='exit'
alias ip='ipconfig getifaddr en0'
alias pp="pnpm"

alias glog-pretty='git log --pretty="%C(Yellow)%h  %C(reset)%ad (%C(Green)%cr%C(reset))%x09 %C(reset)%s %C(Cyan)@%an" --date=short'

function glog-pretty-from {
  command="git log --pretty=\"%C(Yellow)%h  %C(reset)%ad (%C(Green)%cr%C(reset))%x09 %C(reset)%s %C(Cyan)@%an\" --date=short $1...HEAD"
  if [ -n "$2" ]; then
    command+="-- components/$2"
  fi

  eval "$command"
}

function replace-all {
  if [ -n "$2" ]; then
    echo "Usage: replace-all <FIND> <REPLACE>"
  fi

  eval "rg $2 --files-with-matches | xargs sed -i '' -e 's/$2/$3/g'"
}

function y {
	local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
	yazi "$@" --cwd-file="$tmp"
	if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
		builtin cd -- "$cwd"
	fi
	rm -f -- "$tmp"
}
