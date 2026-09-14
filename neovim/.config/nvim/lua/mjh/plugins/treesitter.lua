---@type LazySpec
return {
  --- https://github.com/romus204/tree-sitter-manager.nvim
  'romus204/tree-sitter-manager.nvim',
  dependencies = {}, -- tree-sitter CLI must be installed system-wide
  cmd = { 'TSManager' },
  init = function()
    vim.opt.foldcolumn = '0' -- disable foldcolumn
    vim.opt.foldtext = '' -- display a fold as a normal line with highlighting and no line wrapping
    vim.opt.foldlevelstart = 99 -- start editing a new buffer with all folds open

    -- Safely (the pcall) try to start treesitter
    -- for the current filetype when opening a buffer
    vim.api.nvim_create_autocmd('FileType', {
      callback = function()
        local success, _ = pcall(vim.treesitter.start)

        -- If treesitter is successfully started for the current filetype
        -- set the fold function to treesitter and the method by which to
        -- handle folding to that function
        if success then
          vim.wo[0][0].foldexpr = 'v:lua.vim.treesitter.foldexpr()'
          vim.wo[0][0].foldmethod = 'expr'
        end
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
      'ghostty',
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
