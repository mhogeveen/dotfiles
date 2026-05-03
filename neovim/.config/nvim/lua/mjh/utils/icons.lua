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
  dir_empty = icons.get 'file-directory-outline',
  dir_empty_open = icons.get 'file-directory',
  file_default = icons.get 'file',
  file_symlink = icons.get 'file-symlink-file',
  file_dot_file = icons.get 'file-code',
}

return M
