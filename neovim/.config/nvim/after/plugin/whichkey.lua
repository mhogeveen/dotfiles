local status, whichkey = pcall(require, "which-key")
if not status then
	print("Which-key is not installed")
	return
end

whichkey.setup({
	window = {
		border = "rounded", -- none, single, double, shadow
	},
})

whichkey.register({
	["g"] = {
		name = "LSP",
		D = { "Go to declaration" },
		h = { "Show hover panel" },
		d = { "Go to definition" },
		t = { "Go to type definition" },
		i = { "Go to implementation" },
		s = { "Signature help" },
		S = { "Document symbols" },
		r = { "Show references" },
		R = { "Rename" },
		l = { "Show line diagnostics" },
		f = { "Format" },
		a = { "Code actions" },
	},

	-- Leader
	["<leader>e"] = { "Toggle tree explorer" },
	["<leader>t"] = { "Neogit" },
	["<leader>b"] = { "Buffer list" },
	["<leader>c"] = { "Close buffer" },
	["<leader>f"] = {
		name = "Telescope",
		f = { "Find File" },
		g = { "Git files" },
		r = { "Live grep" },
		b = { "Buffers" },
	},
})
