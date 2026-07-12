# Export environment variables

[[ -f "$ZDOTDIR/env.zsh" ]] && source "$ZDOTDIR/env.zsh"

## Set XDG base directories
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_CACHE_HOME="$HOME/.cache"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_STATE_HOME="$HOME/.local/state"

# Extend path
export PATH="~/opt/bin:$PATH"
export PATH="$HOME/.composer/vendor/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export PATH="$PATH:$HOME/go/bin"
export PATH="$XDG_CONFIG_HOME/emacs/bin:$PATH"

export GPG_TTY="$TTY"
export EDITOR="nvim"
export TERMINAL="wezterm"
export BROWSER="arc"
export MANPAGER="nvim +Man!"
export HISTFILE="$XDG_STATE_HOME/zsh/history"
export HISTSIZE=5000
export SAVEHIST=$HISTSIZE
export HISTDUP="erase"
export KEYTIMEOUT=25
