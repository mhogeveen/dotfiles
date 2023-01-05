local status, treesitter = pcall(require, "nvim-treesitter.configs")
if not status then
	print("Treesitter is not installed")
	return
end

treesitter.setup({
	auto_install = true,
	highlight = { enable = true, additional_vim_regex_highlighting = false },
	incremental_selection = { enable = true },
	indent = { enable = true },
	autopairs = { enable = true },
	textobjects = {
		enable = true,
		select = {
			enable = true,
			lookahead = true,
			keymaps = {
				["af"] = "@function.outer",
				["if"] = "@function.inner",
				["ac"] = "@class.outer",
				["ic"] = { query = "@class.inner", desc = "Select inner part of a class region" },
			},
		},
	},
})
