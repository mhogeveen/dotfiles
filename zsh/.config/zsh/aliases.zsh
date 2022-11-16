# Define aliases.
alias ls='exa -a'
alias c='clear'
alias lg='lazygit -ucd="$HOME/.config/lazygit"'
alias n='nvim'
alias e='exit'
alias grep='rg'
alias :q='exit'
alias ip='ipconfig getifaddr en0'
alias up='brew upgrade && yarn global upgrade' # Upgrade global Packages
alias localphp="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php"
alias localcomposer="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php /Applications/Local.app/Contents/Resources/extraResources/bin/composer/composer.phar"
alias ycu="ncu --packageManager yarn --format group --interactive"
alias tmux="tmux -f ${HOME}/.tmux.conf"
alias rm="rm -i"
alias mv="mv -i"
alias cp="cp -i"

# Wordpress related aliases
alias clone:sb-starter="git clone git@bitbucket.org:socialbrothers/wordpress-starter-theme.git"
alias clone:wp="git clone git@github.com:wordpress/wordpress"

# Switch php versions
alias "php7.4"="valet use php@7.4 && composer global update"
alias "php8.0"="valet use php@8.0 && composer global update"
alias "php8.1"="valet use php && composer global update"
alias php74="valet use php@7.4 && composer global update"
alias php8="valet use php@8.0 && composer global update"
alias php81="valet use php && composer global update"
