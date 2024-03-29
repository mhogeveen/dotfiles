return {
  --- https://github.com/stevearc/conform.nvim
  'stevearc/conform.nvim',
  enabled = true,
  cmd = { 'ConformInfo' },
  event = { 'BufReadPre', 'BufNewFile' },
  opts = function()
    return {
      formatters_by_ft = {
        javascript = { 'prettier' },
        typescript = { 'prettier' },
        javascriptreact = { 'prettier' },
        typescriptreact = { 'prettier' },
        svelte = { 'prettier' },
        css = { { 'stylelint', 'prettier' } },
        scss = { { 'stylelint', 'prettier' } },
        sass = { { 'stylelint', 'prettier' } },
        less = { { 'stylelint', 'prettier' } },
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
        timeout_ms = 500,
      },
      formatters = {
        stylelint = {
          args = { '--fix', '--stdin', '--stdin-filename', '$FILENAME' },
          command = require('conform.util').from_node_modules 'stylelint',
          condition = function(ctx)
            local found = vim.fs.find({
              'stylelint.config.js',
              '.stylelintrc.js',
              'stylelint.config.mjs',
              '.stylelintrc.mjs',
              'stylelint.config.cjs',
              '.stylelintrc.cjs',
              '.stylelintrc.json',
              '.stylelintrc.yaml',
              '.stylelintrc.yml',
              '.stylelintrc',
            }, { upward = true, path = ctx.dirname })[1]

            if found then
              return true
            else
              return false
            end
          end,
        },
      },
    }
  end,
  keys = {
    {
      'gf',
      function()
        require('conform').format { lsp_fallback = true, async = true, timeout_ms = 500 }
      end,
      mode = { 'n', 'v' },
      desc = 'Format file or range (in visual mode)',
    },
  },
}
