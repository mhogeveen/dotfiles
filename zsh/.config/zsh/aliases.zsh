# Define aliases.
alias ls='exa -a'
alias c='clear'
alias lg='lazygit -ucd="$HOME/.config/lazygit"'
alias n='nvim'
alias e='exit'
alias grep='rg'
alias :q='exit'
alias ip='ipconfig getifaddr en0'
alias up='brew upgrade && pnpm up --global --latest' # Upgrade global Packages
alias localphp="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php"
alias localcomposer="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php /Applications/Local.app/Contents/Resources/extraResources/bin/composer/composer.phar"
alias ycu="ncu --packageManager yarn --format group --interactive"
alias tmux="tmux -f ${HOME}/.tmux.conf"
alias rm="rm -i"
alias mv="mv -i"
alias cp="cp -i"

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
