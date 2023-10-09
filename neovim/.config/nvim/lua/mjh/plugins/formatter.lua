return {
  {
    'stevearc/conform.nvim',
    cmd = { 'ConformInfo' },
    event = { 'BufReadPre', 'BufNewFile' },
    opts = {
      formatters_by_ft = {
        javascript = { 'prettierd' },
        typescript = { 'prettierd' },
        javascriptreact = { 'prettierd' },
        typescriptreact = { 'prettierd' },
        svelte = { 'prettierd' },
        css = { { 'stylelint', 'prettierd' } },
        scss = { { 'stylelint', 'prettierd' } },
        sass = { { 'stylelint', 'prettierd' } },
        less = { { 'stylelint', 'prettierd' } },
        html = { 'prettierd' },
        json = { 'prettierd' },
        yaml = { 'prettierd' },
        markdown = { 'prettierd' },
        graphql = { 'prettierd' },
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
    },
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
  },
  {
    'mhartington/formatter.nvim',
    enabled = false,
    event = 'BufEnter',
    config = false,
    init = function()
      local defaults = require 'formatter.defaults'
      local util = require 'formatter.util'

      require('formatter').setup {
        logging = true,
        log_level = vim.log.levels.WARN,
        filetype = {
          html = defaults.prettierd,
          css = defaults.prettierd,
          json = defaults.prettierd,
          jsonc = defaults.prettierd,
          yaml = defaults.prettierd,
          graphql = defaults.prettierd,
          javascript = defaults.prettierd,
          javascriptreact = defaults.prettierd,
          typescript = defaults.prettierd,
          typescriptreact = defaults.prettierd,
          lua = require('formatter.filetypes.lua').stylua,
          rust = require('formatter.filetypes.rust').rustfmt,
          php = function()
            return {
              exe = 'php-cs-fixer',
              args = {
                'fix',
                '--using-cache=no',
              },
              stdin = false,
              ignore_exitcode = true,
            }
          end,
          ['*'] = function()
            -- Do not remove trailing whitespace in markdown files
            if util.get_current_buffer_file_extension() == 'md' then
              return
            end

            util.withl(defaults.sed, '[ \t]*$')
          end,
        },
      }
    end,
  },
}
