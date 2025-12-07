---@type LazySpec
return {
  --- https://github.com/stevearc/conform.nvim
  'stevearc/conform.nvim',
  enabled = true,
  cmd = { 'ConformInfo' },
  event = { 'BufWritePre' },
  ---@module "conform"
  ---@type conform.setupOpts
  opts = {
    formatters_by_ft = {
      javascript = { 'biome-check', 'prettierd', 'prettier', stop_after_first = true },
      typescript = { 'biome-check', 'prettierd', 'prettier', stop_after_first = true },
      javascriptreact = { 'biome-check', 'prettierd', 'prettier', stop_after_first = true },
      typescriptreact = { 'biome-check', 'prettierd', 'prettier', stop_after_first = true },
      svelte = { 'prettierd', 'prettier', stop_after_first = true },
      css = { 'stylelint', 'prettierd', 'prettier', stop_after_first = true },
      scss = { 'stylelint', 'prettierd', 'prettier', stop_after_first = true },
      sass = { 'stylelint', 'prettierd', 'prettier', stop_after_first = true },
      less = { 'stylelint', 'prettierd', 'prettier', stop_after_first = true },
      html = { 'prettierd', 'prettier', stop_after_first = true },
      json = { 'prettierd', 'prettier', stop_after_first = true },
      yaml = { 'prettierd', 'prettier', stop_after_first = true },
      markdown = { 'prettierd', 'prettier', stop_after_first = true },
      graphql = { 'prettierd', 'prettier', stop_after_first = true },
      lua = { 'stylua' },
      rust = { 'rustfmt' },
      php = { 'php_cs_fixer' },
      http = { 'kulala' },
      rest = { 'kulala' },
      terraform = { 'terraform_fmt' },
      ['*'] = { 'trim_newlines', 'trim_whitespace' },
    },
    format_on_save = {
      lsp_fallback = true,
      async = false,
      timeout_ms = 1000,
    },
    formatters = {
      --   stylelint = {
      --     args = { '--fix', '--stdin', '--stdin-filename', '$FILENAME' },
      --     command = require('conform.util').from_node_modules 'stylelint',
      --   },
      kulala = {
        command = 'kulala-fmt',
        args = { 'format', '$FILENAME' },
        stdin = false,
      },
    },
  },
  keys = {
    {
      'gf',
      function()
        require('conform').format { lsp_fallback = true, async = false, timeout_ms = 1000 }
      end,
      mode = { 'n', 'v' },
      desc = 'Format file or range (in visual mode)',
    },
  },
}
