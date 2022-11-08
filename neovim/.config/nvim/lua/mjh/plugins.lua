local ensure_packer = function()
	local fn = vim.fn
	local install_path = fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
	if fn.empty(fn.glob(install_path)) > 0 then
		fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
		vim.cmd([[packadd packer.nvim]])
		return true
	end
	return false
end

-- Autocommand that reloads neovim whenever you save the plugins.lua file
vim.cmd([[
  augroup packer_user_config
    autocmd!
    autocmd BufWritePost plugins.lua source <afile> | PackerSync
  augroup end
]])

local status_packer, packer = pcall(require, "packer")
if not status_packer then
	print("Packer is not installed")
	return
end

local status_impatient, impatient = pcall(require, "impatient")
if not status_impatient then
	return
end

-- Enable impatient profiling (only enable when wanting to view profiling info)
-- impatient.enable_profile()

-- Have packer use a popup window
packer.init({
	display = {
		open_fn = function()
			return require("packer.util").float({ border = "rounded" })
		end,
	},
})

packer.startup(function(use)
	-- Packer
	use({ "wbthomason/packer.nvim" })

	-- Often used requirements
	use({
		"kyazdani42/nvim-web-devicons",
		"nvim-lua/plenary.nvim",
	})

	-- Impatient
	use({ "lewis6991/impatient.nvim" })

	-- Colorschemes
	use({
		"projekt0n/github-nvim-theme",
		"morhetz/gruvbox",
		"rebelot/kanagawa.nvim",
		"folke/tokyonight.nvim",
	})
	use({ "catppuccin/nvim", as = "catppuccin" })

	-- Nvim tree
	use({
		"kyazdani42/nvim-tree.lua",
		tag = "nightly", -- optional, updated every week. (see issue #1193)
	})

	-- Telescope
	use({
		"nvim-telescope/telescope.nvim",
		tag = "0.1.x",
	})
	use({
		"nvim-telescope/telescope-fzf-native.nvim",
		run = "make",
	})

	-- LSP
	use({
		"williamboman/mason.nvim",
		"williamboman/mason-lspconfig.nvim",
		"neovim/nvim-lspconfig",
		"jose-elias-alvarez/null-ls.nvim",
	})

	-- Completion
	use({
		"hrsh7th/nvim-cmp",
		"hrsh7th/cmp-buffer",
		"hrsh7th/cmp-path",
		"hrsh7th/cmp-nvim-lsp",
		"hrsh7th/cmp-nvim-lua",
		"L3MON4D3/LuaSnip",
		"saadparwaiz1/cmp_luasnip",
		"onsails/lspkind.nvim",
		"rafamadriz/friendly-snippets",
	})

	-- Statusline
	use({
		"nvim-lualine/lualine.nvim",
	})

	-- Treesitter
	use({
		"nvim-treesitter/nvim-treesitter",
		"nvim-treesitter/nvim-treesitter-context",
		"nvim-treesitter/playground",
	})

	-- Wakatime
	use({ "wakatime/vim-wakatime" })

	-- Which key
	use({ "folke/which-key.nvim" })

	-- Autopair
	use({ "windwp/nvim-autopairs" })

	-- Startup
	use({ "goolord/alpha-nvim" })

	-- Comment
	use({ "numToStr/Comment.nvim" })

	-- Neogit
	use({ "TimUntersberger/neogit" })

	-- Colorizer
	use({ "NvChad/nvim-colorizer.lua" })

	-- Stabilize
	use({ "luukvbaal/stabilize.nvim" })

	-- Illuminate
	use({ "RRethy/vim-illuminate" })

	-- Gitsigns
	use({ "lewis6991/gitsigns.nvim" })

	-- Modes
	use({
		"mvllow/modes.nvim",
		tag = "v0.2.0",
	})

	-- Fidget
	use({ "j-hui/fidget.nvim" })

	-- Automatically set up your configuration after cloning packer.nvim
	-- Put this at the end after all plugins
	if ensure_packer() then
		packer.sync()
	end
end)
