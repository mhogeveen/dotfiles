---@type LazySpec
return {
  --- https://github.com/ibhagwan/fzf-lua
  'ibhagwan/fzf-lua',
  dependencies = { 'nvim-tree/nvim-web-devicons' },
  keys = {
    {
      'ff',
      function()
        require('fzf-lua').files()
      end,
      mode = 'n',
      desc = 'fzf - find files',
      noremap = true,
      silent = true,
    },
    {
      'fg',
      function()
        require('fzf-lua').git_files()
      end,
      mode = 'n',
      desc = 'fzf - git files',
      noremap = true,
      silent = true,
    },
    {
      'fr',
      function()
        require('fzf-lua').live_grep()
      end,
      mode = 'n',
      desc = 'fzf - grep',
      noremap = true,
      silent = true,
    },
    {
      'fc',
      function()
        require('fzf-lua').grep_cword()
      end,
      mode = 'n',
      desc = 'fzf - grep cword',
      noremap = true,
      silent = true,
    },
    {
      'fb',
      function()
        require('fzf-lua').buffers()
      end,
      mode = 'n',
      desc = 'fzf - buffers',
      noremap = true,
      silent = true,
    },
  },
  cmd = 'FzfLua',
  opts = {
    winopts = {
      border = 'single',
    },
    files = {
      formatter = 'path.filename_first',
    },
    buffers = {
      formatter = 'path.filename_first',
    },
    grep = {
      formatter = 'path.filename_first',
    },
  },
  config = function(_, opts)
    local fzf_lua = require 'fzf-lua'
    fzf_lua.setup(opts)
    fzf_lua.register_ui_select()
  end,
}
