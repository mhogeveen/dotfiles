local status, theme = pcall(require, "catppuccin")
if not status then
	return
end

-- Set Catppuccin flavour (can be set with the :Catppuccin <flavour> cmd on the fly)
vim.g.catppuccin_flavour = "mocha" -- latte, frappe, macchiato, mocha

-- Run theme setup function
theme.setup({
	transparent_background = false,
	styles = {
		comments = { "italic" },
		conditionals = {},
		loops = {},
		functions = { "italic" },
		keywords = {},
		strings = {},
		variables = {},
		numbers = {},
		booleans = {},
		properties = {},
		types = {},
		operators = {},
	},
	integrations = {
		cmp = true,
		nvimtree = true,
		treesitter = true,
		treesitter_context = true,
		telescope = true,
		which_key = true,
		illuminate = true,
		fidget = true,
		gitsigns = true,
		mason = true,
		neogit = true,
	},
	highlight_overrides = {
		mocha = function(colors)
			return {
				FidgetTitle = { fg = colors.blue, bg = colors.none },
				FidgetTask = { fg = colors.blue, bg = colors.none },
			}
		end,
	},
})

-- Set colorscheme with Vim cmd
vim.cmd("colorscheme catppuccin")
