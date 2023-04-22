local ok, lspwindows = pcall(require, "lspconfig.ui.windows")
if not ok then
  print("'lspconfig.ui.windows' not found: 'nvim-lspconfig' is not installed")
  return
end

-- Set window styles
lspwindows.default_options.border = "single"

-- Configure vim diagnostics
vim.diagnostic.config({
  virtual_text = false,
  update_in_insert = true,
  float = {
    border = "single",
  },
})
vim.lsp.handlers["textDocument/hover"] = vim.lsp.with(vim.lsp.handlers.hover, { border = "single" })
vim.lsp.handlers["textDocument/signatureHelp"] = vim.lsp.with(vim.lsp.handlers.signature_help, { border = "single" })
