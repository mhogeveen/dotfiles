local status, null_ls = pcall(require, "null-ls")
if not status then
  print("'null-ls' is not installed")
  return
end

local augroup = vim.api.nvim_create_augroup("LspFormatting", {})

null_ls.setup({
  sources = {
    null_ls.builtins.diagnostics.eslint,
    null_ls.builtins.diagnostics.stylelint,
    null_ls.builtins.formatting.stylua,
    null_ls.builtins.formatting.phpcsfixer.with({
      extra_args = { "--using-cache=no" },
    }),
    null_ls.builtins.formatting.rustfmt,
    null_ls.builtins.formatting.prettierd,
    null_ls.builtins.formatting.stylelint,
    null_ls.builtins.formatting.trim_newlines.with({
      disabled_filetypes = { "md" },
    }),
    null_ls.builtins.formatting.trim_whitespace.with({
      disabled_filetypes = { "md" },
    }),
  },
  on_attach = function(client, bufnr)
    if client.supports_method("textDocument/formatting") then
      vim.api.nvim_clear_autocmds({ group = augroup, buffer = bufnr })
      vim.api.nvim_create_autocmd("BufWritePre", {
        group = augroup,
        buffer = bufnr,
        callback = function()
          vim.lsp.buf.format({
            async = false,
            bufnr = bufnr,
            filter = function()
              return client.name == "null-ls"
            end,
          })
        end,
      })
    end
  end,
})
