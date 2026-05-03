---@type LazySpec
return {
  {
    'romus204/tree-sitter-manager.nvim',
    dependencies = {}, -- tree-sitter CLI must be installed system-wide
    init = function()
      vim.api.nvim_create_autocmd('FileType', {
        callback = function()
          pcall(vim.treesitter.start)
        end,
      })
    end,
    opts = {
      ensure_installed = {
        'bash',
        'bicep',
        'c',
        'comment',
        'css',
        'dockerfile',
        'dtd',
        'ecma',
        'editorconfig',
        'git_config',
        'git_rebase',
        'gitattributes',
        'gitcommit',
        'gitignore',
        'go',
        'gomod',
        'gosum',
        'gowork',
        'graphql',
        'hcl',
        'html',
        'html_tags',
        'http',
        'javascript',
        'jq',
        'json',
        'jsx',
        'lua',
        'luadoc',
        'luap',
        'make',
        'markdown',
        'markdown_inline',
        'php',
        'php_only',
        'phpdoc',
        'proto',
        'python',
        'query',
        'regex',
        'rust',
        'scss',
        'ssh_config',
        'svelte',
        'terraform',
        'toml',
        'tsx',
        'typescript',
        'vim',
        'vimdoc',
        'vue',
        'xml',
        'yaml',
        'zig',
        'zsh',
      }, -- list of parsers to install at the start of a neovim session
      border = 'single', -- border style for the window (e.g. "rounded", "single"), if nil, use the default border style defined by 'vim.o.winborder'. See :h 'winborder' for more info.
      auto_install = true, -- if enabled, install missing parsers when editing a new file
      highlight = false, -- treesitter highlighting is enabled by default
      languages = {}, -- override or add new parser sources
      parser_dir = vim.fn.stdpath 'data' .. '/site/parser',
      query_dir = vim.fn.stdpath 'data' .. '/site/queries',
    },
  },
  {
    --- https://github.com/nvim-treesitter/nvim-treesitter
    'nvim-treesitter/nvim-treesitter',
    enabled = false,
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
        'graphql',
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
  },
}
