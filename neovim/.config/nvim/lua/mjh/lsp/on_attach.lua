local function lsp_highlight_document(client)
	-- Set autocommands conditional on server_capabilities
	if client.server_capabilities.document_highlight then
		vim.api.nvim_exec(
			[[
      augroup lsp_document_highlight
        autocmd! * <buffer>
        autocmd CursorHold <buffer> lua vim.lsp.buf.document_highlight()
        autocmd CursorMoved <buffer> lua vim.lsp.buf.clear_references()
      augroup END
    ]],
			false
		)
	end
end

local function lsp_keymaps(bufnr)
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
end

return function(client, bufnr)
	lsp_highlight_document(client)
	lsp_keymaps(bufnr)
end
