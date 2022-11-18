# Set shell options
setopt autocd       # Using the AUTOCD option, you can simply type the name of a directory, and it will become the current directory.
setopt globdots     # GLOBDOTS lets files beginning with a . be matched without explicitly specifying the dot.
setopt appendhistory

# No highlight after pasting
zle_highlight=('paste:none')
