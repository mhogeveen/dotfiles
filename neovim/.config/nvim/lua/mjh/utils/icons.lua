local icons = require 'nvim-nonicons'

local M = {
  gitsigns = {
    add = '+',
    change = '~',
    delete = '-',
    topdelete = '‾',
    changedelete = '~',
    untracked = '┆',
  },

  dir_open = icons.get 'file-directory-open-fill',
  dir_closed = icons.get 'file-directory-fill',
  file_default = icons.get 'file',
  file_symlink = icons.get 'file-symlink-file',
  file_dot_file = icons.get 'file-code',
}

return M
