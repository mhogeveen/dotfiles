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

zstyle ':vcs_info:*' check-for-changes true # Enables use of %u and %c
zstyle ':vcs_info:*' use-simple true # If available use simple method of collecting info (better perf)
zstyle ':vcs_info:git:*' stagedstr 'S' # Set string shown by %c
zstyle ':vcs_info:git:*' unstagedstr 'U' # Set string shown by %u
zstyle ':vcs_info:git:*' formats '(%F{magenta}%s:%b%f)%F{red}%u%f%F{green}%c%f'

setopt PROMPT_SUBST

# For available prompt escape sequences see:
#   $ man zshmisc
#   under section named 'Simple Prompt Escapes'

# Left Prompt
PROMPT='%F{green}$USER@$HOST%f '

# If the current path has at least 4 elements relative to the root directory
#   print `~/.../<last two path elements>`
# else
#   print '<all path elements>'
PROMPT+='%F{blue}%(4~|~/.../%2~|%~)%f '
# Add (<vcs>:<branch>)<has unstaged><has staged> (when in git project)
PROMPT+=$'${vcs_info_msg_0_}\n'
# Print '>' in green if previous command was succesful,
# otherwise print '>' in red
PROMPT+='%(?.%F{green}>.%F{red}>)%f '

# Right Prompt
# RPROMPT=''
