# Autoload functions
autoload -Uz compinit
compinit -d "$XDG_CACHE_HOME/zsh/zcompdump"

# Set PATH, MANPATH, etc., for Homebrew.
# Needs to be 'eval'ed before any package eval's installed by Homebrew
[[ -f "/opt/homebrew/bin/brew" ]] && eval "$(/opt/homebrew/bin/brew shellenv)"

# Add fnm to shell
type fnm &> /dev/null && eval "$(fnm env --use-on-cd --shell=zsh)"

# Add zoxide to shell
type zoxide &> /dev/null && eval "$(zoxide init zsh)"

# Add fzf to shell
type fzf &> /dev/null && eval "$(fzf --zsh)"

if [[ -f /opt/homebrew/opt/fzf/shell/key-bindings.zsh ]]; then
  source /opt/homebrew/opt/fzf/shell/key-bindings.zsh
  source /opt/homebrew/opt/fzf/shell/completion.zsh
fi

# Add cargo to shell
[[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"

# pnpm
export PNPM_HOME="/Users/$USER/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end

# Source useful # Normal files to source
[[ -f "$ZDOTDIR/plugins.zsh" ]] && source "$ZDOTDIR/plugins.zsh"
[[ -f "$ZDOTDIR/prompt.zsh" ]] && source "$ZDOTDIR/prompt.zsh"
[[ -f "$ZDOTDIR/options.zsh" ]] && source "$ZDOTDIR/options.zsh"
[[ -f "$ZDOTDIR/aliases.zsh" ]] && source "$ZDOTDIR/aliases.zsh"
[[ -f "$ZDOTDIR/keymaps.zsh" ]] && source "$ZDOTDIR/keymaps.zsh"
[[ -f "$ZDOTDIR/fzf.zsh" ]] && source "$ZDOTDIR/fzf.zsh"
