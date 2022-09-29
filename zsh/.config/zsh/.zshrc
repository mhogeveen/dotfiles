# Source useful functions
source "$ZDOTDIR/zsh-functions"

# Normal files to source
zsh_add_file "zsh-exports"
zsh_add_file "zsh-options"
zsh_add_file "zsh-aliases"
zsh_add_file "zsh-vim-mode"

# Plugins
zsh_add_plugin "zsh-users/zsh-autosuggestions"
zsh_add_plugin "zsh-users/zsh-syntax-highlighting"
zsh_add_plugin "hlissner/zsh-autopair"

# Add fnm to shell
eval "$(fnm env --use-on-cd)"

# Add zoxide to shell
eval "$(zoxide init zsh)"

# Add oh-my-posh to shell
eval "$(oh-my-posh init zsh --config ~/.config/oh-my-posh/.theme.omp.json)"
enable_poshtransientprompt
