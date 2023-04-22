local status_ok, cmp_nvim_lsp = pcall(require, "cmp_nvim_lsp")
if not status_ok then
  return
end

-- Set up capabilities
local capabilities = vim.lsp.protocol.make_client_capabilities()
capabilities.textDocument.completion.completionItem.snippetSupport = true

-- Configure default configuration table
local default_config = {
  on_attach = function(_, bufnr)
    local opts = { noremap = true, silent = true }
    local function set_buf_keymap(keymap, command)
      vim.api.nvim_buf_set_keymap(bufnr, "n", keymap, command, opts)
    end

    set_buf_keymap("gD", ":lua vim.lsp.buf.declaration()<CR>")
    set_buf_keymap("gh", ":lua vim.lsp.buf.hover()<CR>")
    set_buf_keymap("gd", ":lua vim.lsp.buf.definition()<CR>")
    set_buf_keymap("gt", ":lua vim.lsp.buf.type_definition()<CR>")
    set_buf_keymap("gi", ":Telescope lsp_implementations<CR>")
    set_buf_keymap("gs", ":lua vim.lsp.buf.signature_help()<CR>")
    set_buf_keymap("gS", ":Telescope lsp_document_symbols<CR>")
    set_buf_keymap("gr", ":Telescope lsp_references<CR>")
    set_buf_keymap("gR", ":lua vim.lsp.buf.rename()<CR>")
    set_buf_keymap("[d", ":lua vim.diagnostic.goto_prev()<CR>")
    set_buf_keymap("gl", ":lua vim.diagnostic.open_float()<CR>")
    set_buf_keymap("]d", ":lua vim.diagnostic.goto_next()<CR>")
    set_buf_keymap("gf", ":lua vim.lsp.buf.format()<CR>")
    set_buf_keymap("ga", ":lua vim.lsp.buf.code_action()<CR>")
  end,
  capabilities = cmp_nvim_lsp.default_capabilities(capabilities)
}

return function(custom_lsp_config)
  return vim.tbl_deep_extend("force", custom_lsp_config, default_config)
end
