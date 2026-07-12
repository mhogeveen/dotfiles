# Set shell options
setopt numeric_glob_sort      # sort file10 after file9, not after file1
setopt nobeep                 # Do not beep
setopt autocd                 # Using the AUTOCD option, you can simply type the name of a directory, and it will become the current directory.
setopt globdots               # GLOBDOTS lets files beginning with a . be matched without explicitly specifying the dot.
setopt appendhistory          # Append commands to history instead of overwriting
setopt sharehistory           # Share history across zsh sessions
setopt hist_ignore_all_dups
setopt hist_save_no_dups
setopt hist_ignore_dups
setopt hist_find_no_dups
setopt hist_ignore_space
setopt hist_expire_dups_first

zle_highlight=('paste:none')  # No highlight after pasting

# Autocomplete case insensitively
zstyle ':completion:*' matcher-list 'm:{a-z}={a-zA-Z}'

# Enable interactive completion menu select
zstyle ':completion:*' menu select

# Apply ls colors to completion options
zstyle ':completion:*' list-colors ${(s.:.)LS_COLORS}
