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
