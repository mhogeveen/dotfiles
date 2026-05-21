---@type LazySpec
return {
  --- https://github.com/romus204/tree-sitter-manager.nvim
  'romus204/tree-sitter-manager.nvim',
  dependencies = {}, -- tree-sitter CLI must be installed system-wide
  cmd = { 'TSManager' },
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
}
