#!/bin/zsh

autoload -Uz vcs_info
precmd() { vcs_info }

# Available format strings for vcs:
# - %s The current version control system, like git or svn
# - %r The name of the root directory of the repository
# - %S The current path relative to the repository root directory
# - %b Branch information, like master
# - %m In case of Git, show information about stashes
# - %u Show unstaged changes in the repository
# - %c Show staged changes in the repository

# zstyle ':vcs_info:*' enable git svn # Enables use of %s for listed vcs'
zstyle ':vcs_info:*' check-for-changes true # Enables use of %u and %c
zstyle ':vcs_info:git:*' formats '(%F{magenta}%b%f)%F{red}%u%f%F{green}%c%f'

setopt PROMPT_SUBST

# ---------------------------
# Left Prompt
# ---------------------------

PROMPT=''

# Print working directory or ~ if in home
PROMPT+='%F{blue}%1~%f '
# Add current git branch (if in git project)
PROMPT+=$'${vcs_info_msg_0_}\n'
# Print double chevron in green if previous command was succesful,
# otherwise print double chevron in red
PROMPT+='%(?.%F{green}.%F{red})%f '
#  (old icon which doesn't work anymore with BlexMono patched font)

# ---------------------------
# Right Prompt
# ---------------------------

# RPROMPT=''
