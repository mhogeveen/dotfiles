# Extend PATH.
PATH=~/opt/bin:$PATH

# Export environment variables.
export GPG_TTY="$TTY"
export PATH="$HOME/.composer/vendor/bin:$PATH"
export PATH="$HOME/bin:$PATH"
export EDITOR="nvim"
export TERMINAL="wezterm"
export BROWSER="arc"
export MANPAGER="nvim +Man!"
export HOMEBREW_BUNDLE_FILE="$HOME/.dotfiles/homebrew/Brewfile"
export HISTSIZE=5000
export SAVEHIST=$HISTSIZE
export HISTDUP="erase"
export KEYTIMEOUT=25

export FZF_DEFAULT_OPTS=" \
--color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8 \
--color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc \
--color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8 \
--height=~50%"
