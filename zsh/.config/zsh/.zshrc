# Check if Zap is installed, if so: source
# https://github.com/zap-zsh/zap
[ -f "$HOME/.local/share/zap/zap.zsh" ] && source "$HOME/.local/share/zap/zap.zsh"

# Source useful # Normal files to source
[ -f "$ZDOTDIR/prompt.zsh" ] && source "$ZDOTDIR/prompt.zsh"
[ -f "$ZDOTDIR/exports.zsh" ] && source "$ZDOTDIR/exports.zsh"
[ -f "$ZDOTDIR/env.zsh" ] && source "$ZDOTDIR/env.zsh"
[ -f "$ZDOTDIR/options.zsh" ] && source "$ZDOTDIR/options.zsh"
[ -f "$ZDOTDIR/aliases.zsh" ] && source "$ZDOTDIR/aliases.zsh"
[ -f "$ZDOTDIR/keymaps.zsh" ] && source "$ZDOTDIR/keymaps.zsh"
[ -f "$ZDOTDIR/cursor.zsh" ] && source "$ZDOTDIR/cursor.zsh"

# Plugins
plug "zsh-users/zsh-autosuggestions"
plug "zsh-users/zsh-syntax-highlighting"
plug "zsh-users/zsh-completions"
plug "Aloxaf/fzf-tab"

# Autoload functions
autoload -Uz compinit && compinit

# Add fnm to shell
eval "$(fnm env --use-on-cd)"

# Add zoxide to shell
eval "$(zoxide init zsh)"

# Add fzf to shell
eval "$(fzf --zsh)"

# pnpm
export PNPM_HOME="/Users/$USER/Library/pnpm"
case ":$PATH:" in
  *":$PNPM_HOME:"*) ;;
  *) export PATH="$PNPM_HOME:$PATH" ;;
esac
# pnpm end
