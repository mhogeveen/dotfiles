---@type LazySpec
return {
  --- https://github.com/stevearc/conform.nvim
  'stevearc/conform.nvim',
  enabled = true,
  cmd = { 'ConformInfo' },
  event = { 'BufWritePre' },
  opts = function()
    ---@module "conform"
    ---@type conform.setupOpts
    return {
      formatters_by_ft = {
        javascript = { 'prettierd', 'prettier', 'biome', stop_after_first = true },
        typescript = { 'prettierd', 'prettier', 'biome', stop_after_first = true },
        javascriptreact = { 'prettierd', 'prettier', 'biome', stop_after_first = true },
        typescriptreact = { 'prettierd', 'prettier', 'biome', stop_after_first = true },
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
        ['*'] = { 'trim_newlines', 'trim_whitespace' },
      },
      format_on_save = {
        lsp_fallback = true,
        async = false,
        timeout_ms = 1000,
      },
      -- formatters = {
      --   stylelint = {
      --     args = { '--fix', '--stdin', '--stdin-filename', '$FILENAME' },
      --     command = require('conform.util').from_node_modules 'stylelint',
      --   },
      -- },
    }
  end,
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
