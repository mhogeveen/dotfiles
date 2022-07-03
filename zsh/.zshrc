# Start shell with tmux
# if [ "$TMUX" = "" ]; then tmux; fi

# Extend PATH.
path=(~/bin $path)

# Export environment variables.
export GPG_TTY=$TTY
export PATH="$HOME/.composer/vendor/bin:$PATH"
export PATH="$HOME/bin:$PATH"

# Set shell options
setopt autocd       # Using the AUTOCD option, you can simply type the name of a directory, and it will become the current directory.
setopt globdots     # GLOBDOTS lets files beginning with a . be matched without explicitly specifying the dot.

# Autoload functions.
autoload -Uz compinit
compinit

# Source antigen (ZSH package manager (kinda))
# https://github.com/zsh-users/antigen
source /opt/homebrew/share/antigen/antigen.zsh
antigen init ~/.antigenrc

# Define aliases.
alias tree='tree -a -I .git'
alias ls='exa --icons --all'
alias c='clear'
alias lg='lazygit'
alias e='exit'
alias localphp="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php"
alias localcomposer="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php /Applications/Local.app/Contents/Resources/extraResources/bin/composer/composer.phar"

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

# Add fnm to shell
eval "$(fnm env --use-on-cd)"

# Add zoxide to shell
eval "$(zoxide init zsh)"

# Add oh-my-posh to shell
eval "$(oh-my-posh init zsh --config ~/.config/oh-my-posh/.theme.omp.json)"
enable_poshtransientprompt

# Run 'macchina' command on startup.
# macchina