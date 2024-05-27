# Set shell options
setopt autocd                 # Using the AUTOCD option, you can simply type the name of a directory, and it will become the current directory.
setopt globdots               # GLOBDOTS lets files beginning with a . be matched without explicitly specifying the dot.
setopt appendhistory          # Append commands to history instead of overwriting
setopt sharehistory           # Share history across zsh sessions
setopt hist_ignore_all_dups
setopt hist_save_no_dups
setopt hist_ignore_dups
setopt hist_find_no_dups

zle_highlight=('paste:none')  # No highlight after pasting

# Autocomplete case insensitively
zstyle ':completion:*' matcher-list 'm:{a-z}={a-zA-Z}'

# Don't use default menu for completions (fzf is used via fzf-tab)
zstyle ':completion:*' menu no

# Apply ls colors to completion options
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
