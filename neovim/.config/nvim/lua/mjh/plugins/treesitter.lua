---@type LazySpec
return {
  --- https://github.com/nvim-treesitter/nvim-treesitter
  'nvim-treesitter/nvim-treesitter',
  branch = 'main',
  build = ':TSUpdate',
  cmd = {
    'TSInstall',
    'TSInstallFromGrammar',
    'TSLog',
    'TSUninstall',
    'TSUpdate',
  },
  main = 'nvim-treesitter',
  dependencies = {
    {
      'nvim-treesitter/nvim-treesitter-textobjects',
      branch = 'main',
    },
  },
  init = function()
    local ensure_installed = {
      'bash',
      'c',
      'css',
      'go',
      'gomod',
      'gosum',
      'gowork',
      'html',
      'javascript',
      'json',
      'latex',
      'lua',
      'luadoc',
      'luap',
      'markdown',
      'markdown_inline',
      'php',
      'proto',
      'python',
      'query',
      'regex',
      'rust',
      'scss',
      'svelte',
      'swift',
      'terraform',
      'tsx',
      'typescript',
      'vim',
      'vimdoc',
      'vue',
      'yaml',
      'zig',
    }
    local already_installed = require('nvim-treesitter.config').get_installed()
    local parsers_to_install = vim
      .iter(ensure_installed)
      :filter(function(parser)
        return not vim.tbl_contains(already_installed, parser)
      end)
      :totable()
    require('nvim-treesitter').install(parsers_to_install)

    vim.api.nvim_create_autocmd('FileType', {
      callback = function()
        -- Enable treesitter highlighting and disable regex syntax
        pcall(vim.treesitter.start)
        -- Enable treesitter-based indentation
        vim.bo.indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
      end,
    })
  end,
  opts = {
    auto_install = true,
    incremental_selection = { enable = true },
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
}
