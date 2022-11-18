# Extend PATH.
PATH=~/opt/bin:$PATH

# Export environment variables.
export GPG_TTY="$TTY"
export PATH="$HOME/.composer/vendor/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export EDITOR="nvim"
export TERMINAL="kitty"
export BROWSER="arc"
export MANPAGER="nvim +Man!"

# Autoload functions.
autoload -Uz compinit
compinit