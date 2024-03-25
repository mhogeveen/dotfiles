return {
  --- https://github.com/nvim-treesitter/nvim-treesitter
  'nvim-treesitter/nvim-treesitter',
  build = ':TSUpdate',
  event = 'VeryLazy',
  cmd = { 'TSUpdateSync', 'TSUpdate', 'TSInstall' },
  dependencies = {
    { 'nvim-treesitter/nvim-treesitter-textobjects' },
  },
  init = function(plugin)
    require('lazy.core.loader').add_to_rtp(plugin)
    require 'nvim-treesitter.query_predicates'
  end,
  opts = {
    auto_install = true,
    highlight = { enable = true, additional_vim_regex_highlighting = false },
    incremental_selection = { enable = true },
    indent = { enable = true },
    autopairs = { enable = true },
    textobjects = {
      enable = true,
      select = {
        enable = true,
        lookahead = true,
        keymaps = {
          ['af'] = '@function.outer',
          ['if'] = '@function.inner',
          ['ac'] = '@class.outer',
          ['ic'] = '@class.inner',
        },
      },
    },
  },
  config = function(_, opts)
    require('nvim-treesitter.configs').setup(opts)
  end,
}
