# Extend PATH.
PATH=~/opt/bin:$PATH

# Export environment variables.
export GPG_TTY=$TTY
export PATH="$HOME/.composer/vendor/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export MANPAGER="nvim +Man!"

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

# fzf-tab config
export FZF_COMPLETION_OPTS='--border --info=inline'
zstyle ':completion:*:git-checkout:*' sort false
zstyle ':completion:*:descriptions' format '[%d]'
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
zstyle ':fzf-tab:complete:cd:*' fzf-preview 'exa -1 --color=always $realpath'
zstyle ':fzf-tab:complete:cd:*' popup-pad 30 0
zstyle ':fzf-tab:*' switch-group ',' '.'
zstyle ':fzf-tab:*' fzf-bindings 'space:accept'
zstyle ':fzf-tab:*' single-group color header prefix

# Define aliases.
alias ls='exa -lah --group-directories-first --no-time --git --sort=name'
alias c='clear'
alias lg='lazygit'
alias n='nvim'
alias e='exit'
alias grep='rg'
alias :q='exit'
alias ip='ipconfig getifaddr en0'
alias localphp="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php"
alias localcomposer="${HOME}/Library/Application\ Support/Local/lightning-services/php-8.0.0+2/bin/darwin/bin/php /Applications/Local.app/Contents/Resources/extraResources/bin/composer/composer.phar"
alias ycu="ncu --packageManager yarn --format group --interactive"
alias tmux="tmux -f ${HOME}/.tmux.conf"

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
