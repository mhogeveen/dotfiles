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
        javascript = { 'prettier' },
        typescript = { 'prettier' },
        javascriptreact = { 'prettier' },
        typescriptreact = { 'prettier' },
        svelte = { 'prettier' },
        css = { 'stylelint', 'prettier', stop_after_first = true },
        scss = { 'stylelint', 'prettier', stop_after_first = true },
        sass = { 'stylelint', 'prettier', stop_after_first = true },
        less = { 'stylelint', 'prettier', stop_after_first = true },
        html = { 'prettier' },
        json = { 'prettier' },
        yaml = { 'prettier' },
        markdown = { 'prettier' },
        graphql = { 'prettier' },
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
      formatters = {
        stylelint = {
          args = { '--fix', '--stdin', '--stdin-filename', '$FILENAME' },
          command = require('conform.util').from_node_modules 'stylelint',
        },
      },
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
