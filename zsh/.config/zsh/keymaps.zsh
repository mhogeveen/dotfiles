# Cursor shape per vi mode
ZVM_INSERT_MODE_CURSOR=$ZVM_CURSOR_BEAM
ZVM_NORMAL_MODE_CURSOR=$ZVM_CURSOR_BLOCK
ZVM_VISUAL_MODE_CURSOR=$ZVM_CURSOR_BLOCK

# Disable command mode line highlight
ZVM_VI_HIGHLIGHT_BACKGROUND=none
ZVM_VI_HIGHLIGHT_FOREGROUND=none
ZVM_VI_HIGHLIGHT_EXTRASTYLE=none

# zsh-vi-mode resets all bindings on init, so custom bindings
# must be registered via this hook to survive.
zvm_after_init() {
  # Ctrl+F -> fzf file picker (no hidden files)
  bindkey '^f' _fzf_file_no_hidden

  # CTRL+y -> accept the autosuggest option currently highlighted
  bindkey '^y' autosuggest-accept

  # Ctrl+p/Ctrl+n -> history search by substring
  bindkey '^p' history-substring-search-up
  bindkey '^n' history-substring-search-down
}
