export FZF_DEFAULT_COMMAND='fd --type f --hidden --strip-cwd-prefix'  # strip-cwd-prefix removes the leading ./ from results

# Ctrl-T uses fd
export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"

# UI
# export FZF_DEFAULT_OPTS=" \
# --color=fg:#CDD6F4,header:#F38BA8,info:#CBA6F7,pointer:#F5E0DC \
# --color=marker:#B4BEFE,fg+:#CDD6F4,prompt:#CBA6F7,hl+:#F38BA8 \
# --color=bg+:#313244,bg:#1E1E2E,spinner:#F5E0DC,hl:#F38BA8 \
# --color=selected-bg:#45475A \
# --color=border:#6C7086,label:#CDD6F4
# --style=minimal \
# --height=~50% \
# --layout=reverse"

export FZF_DEFAULT_OPTS="$FZF_DEFAULT_OPTS \
  --color=bg+:#201b19 \
  --color=bg:#171311 \
  --color=border:#362f2c \
  --color=fg:#a09384 \
  --color=fg+:#e6d5c2 \
  --color=gutter:#171311 \
  --color=header:#4e89a2 \
  --color=hl:#ea9875 \
  --color=hl+:#fcba81 \
  --color=info:#73665b \
  --color=marker:#43b16a \
  --color=pointer:#ea9875 \
  --color=prompt:#d1766e \
  --color=query:#e6d5c2 \
  --color=scrollbar:#362f2c \
  --color=separator:#362f2c \
  --color=spinner:#ea9875 \
  --style=minimal \
  --height=~50% \
  --layout=reverse"

export _FZF_PREVIEW_CMD='bat --color=always --style=plain,numbers --line-range=:500 {}'
export FZF_CTRL_T_OPTS="--preview '$_FZF_PREVIEW_CMD'"

# Ctrl+F: file picker excluding hidden files
_fzf_file_no_hidden() {
  local cmd result
  cmd="${FZF_DEFAULT_COMMAND/--hidden /}"
  result=$(eval "${cmd:-find . -type f}" | fzf --preview "$_FZF_PREVIEW_CMD") \
    && LBUFFER+="$result"  # LBUFFER is the text left of the cursor
  zle reset-prompt
}
zle -N _fzf_file_no_hidden
