return {
  {
    --- https://github.com/nvimtools/none-ls.nvim
    'nvimtools/none-ls.nvim',
    cmd = { 'NullLsInfo', 'NullLsLog' },
    event = { 'BufReadPre', 'BufNewFile' },
    opts = function()
      local null_ls = require 'null-ls'
      local augroup = vim.api.nvim_create_augroup('LspFormatting', {})

      return {
        sources = {
          null_ls.builtins.diagnostics.eslint,
          null_ls.builtins.diagnostics.stylelint,
          null_ls.builtins.formatting.stylua,
          --- rustfmt is installed via rustup
          null_ls.builtins.formatting.rustfmt,
          null_ls.builtins.formatting.prettierd,
          null_ls.builtins.formatting.stylelint,
          null_ls.builtins.formatting.trim_newlines.with {
            disabled_filetypes = { 'md' },
          },
          null_ls.builtins.formatting.trim_whitespace.with {
            disabled_filetypes = { 'md' },
          },
        },
        on_attach = function(client, bufnr)
          if client.supports_method 'textDocument/formatting' then
            vim.api.nvim_clear_autocmds { group = augroup, buffer = bufnr }
            vim.api.nvim_create_autocmd('BufWritePre', {
              group = augroup,
              buffer = bufnr,
              callback = function()
                vim.lsp.buf.format {
                  async = false,
                  bufnr = bufnr,
                  filter = function()
                    return client.name == 'null-ls'
                  end,
                }
              end,
            })
          end
        end,
      }
    end,
  },
  {
    --- https://github.com/stevearc/conform.nvim
    'stevearc/conform.nvim',
    enabled = false,
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
}
