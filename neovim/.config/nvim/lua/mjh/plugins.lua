local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"
if not vim.loop.fs_stat(lazypath) then
  vim.fn.system({
    "git",
    "clone",
    "--filter=blob:none",
    "https://github.com/folke/lazy.nvim.git",
    "--branch=stable", -- latest stable release
    lazypath,
  })
end
vim.opt.rtp:prepend(lazypath)

local status_packer, lazy = pcall(require, "lazy")
if not status_packer then
  print("Lazy is not installed")
  return
end

lazy.setup({
  { "kyazdani42/nvim-web-devicons" },
  { "nvim-lua/plenary.nvim" },
  { "catppuccin/nvim", as = "catppuccin", lazy = false, priority = 1000 },
  { "kyazdani42/nvim-tree.lua", tag = "nightly" },
  {
    "nvim-telescope/telescope.nvim",
    branch = "0.1.x",
    dependencies = { "telescope-fzf-native.nvim", "telescope-ui-select.nvim" },
    cmd = "Telescope",
  },
  { "nvim-telescope/telescope-fzf-native.nvim", build = "make" },
  { "nvim-telescope/telescope-ui-select.nvim" },
  { "neovim/nvim-lspconfig", dependencies = { "mason-lspconfig.nvim" } },
  { "williamboman/mason.nvim" },
  { "williamboman/mason-lspconfig.nvim" },
  { "jose-elias-alvarez/null-ls.nvim" },
  {
    "hrsh7th/nvim-cmp",
    event = "InsertEnter",
    dependencies = {
      "cmp-buffer",
      "cmp-path",
      "cmp-nvim-lsp",
      "cmp-nvim-lua",
      "cmp_luasnip",
    },
  },
  { "hrsh7th/cmp-buffer" },
  { "hrsh7th/cmp-path" },
  { "hrsh7th/cmp-nvim-lsp" },
  { "hrsh7th/cmp-nvim-lua" },
  { "saadparwaiz1/cmp_luasnip" },
  { "L3MON4D3/LuaSnip", dependencies = {
    "friendly-snippets",
  } },
  { "rafamadriz/friendly-snippets" },
  { "onsails/lspkind.nvim" },
  { "nvim-lualine/lualine.nvim" },
  { "nvim-treesitter/nvim-treesitter", build = ":TSUpdate" },
  { "nvim-treesitter/nvim-treesitter-context" },
  { "nvim-treesitter/nvim-treesitter-textobjects" },
  { "nvim-treesitter/playground" },
  { "wakatime/vim-wakatime" },
  { "folke/which-key.nvim", lazy = true },
  { "windwp/nvim-autopairs" },
  { "goolord/alpha-nvim" },
  { "numToStr/Comment.nvim" },
  { "TimUntersberger/neogit" },
  { "NvChad/nvim-colorizer.lua" },
  { "luukvbaal/stabilize.nvim" },
  { "RRethy/vim-illuminate" },
  { "lewis6991/gitsigns.nvim" },
  { "j-hui/fidget.nvim" },
  { "karb94/neoscroll.nvim" },
  { "mhogeveen/reach.nvim" },
  { "famiu/bufdelete.nvim" },
}, {
  lockfile = "~/.dotfiles/neovim/.config/nvim/lazy-lock.json",
})
