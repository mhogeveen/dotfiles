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
  { "folke/lazy.nvim",            tag = "stable" },
  { "nvim-tree/nvim-web-devicons" },
  { "nvim-lua/plenary.nvim" },
  {
    "catppuccin/nvim",
    name = "catppuccin",
    lazy = false,
    priority = 1000,
  },
  {
    "theblob42/drex.nvim",
    cmd = { "DrexDrawerOpen", "DrexDrawerClose", "DrexDrawerToggle" },
    event = "VeryLazy",
  },
  {
    "nvim-telescope/telescope.nvim",
    branch = "0.1.x",
    lazy = true,
    dependencies = { "telescope-fzf-native.nvim", "telescope-ui-select.nvim" },
    cmd = "Telescope",
  },
  { "nvim-telescope/telescope-fzf-native.nvim", build = "make", lazy = true },
  { "nvim-telescope/telescope-ui-select.nvim",  lazy = true },
  { "neovim/nvim-lspconfig",                    lazy = true },
  { "jose-elias-alvarez/null-ls.nvim",          lazy = true },
  {
    "hrsh7th/nvim-cmp",
    dependencies = {
      "cmp-buffer",
      "cmp-path",
      "cmp-nvim-lsp",
      "cmp-nvim-lua",
      "cmp_luasnip",
    },
  },
  { "hrsh7th/cmp-buffer",   lazy = true },
  { "hrsh7th/cmp-path",     lazy = true },
  { "hrsh7th/cmp-nvim-lsp", lazy = true },
  { "hrsh7th/cmp-nvim-lua", lazy = true },
  {
    "tzachar/cmp-tabnine",
    build = "./install.sh",
    dependencies = { "hrsh7th/nvim-cmp" },
  },
  { "saadparwaiz1/cmp_luasnip",                    lazy = true },
  { "L3MON4D3/LuaSnip",                            dependencies = { "friendly-snippets" } },
  { "rafamadriz/friendly-snippets",                lazy = true },
  { "onsails/lspkind.nvim" },
  { "simrat39/symbols-outline.nvim",               event = "VeryLazy" },
  { "petertriho/nvim-scrollbar",                   event = "VeryLazy" },
  { "nvim-lualine/lualine.nvim" },
  { "nvim-treesitter/nvim-treesitter",             build = ":TSUpdate" },
  { "nvim-treesitter/nvim-treesitter-context",     dependencies = { "nvim-treesitter/nvim-treesitter" } },
  { "nvim-treesitter/nvim-treesitter-textobjects", dependencies = { "nvim-treesitter/nvim-treesitter" } },
  -- { "nvim-treesitter/playground" },
  { "wakatime/vim-wakatime" },
  { "folke/which-key.nvim",                        event = "VeryLazy" },
  { "windwp/nvim-autopairs",                       event = "InsertEnter" },
  { "goolord/alpha-nvim" },
  { "numToStr/Comment.nvim",                       event = "BufRead" },
  { "NvChad/nvim-colorizer.lua",                   event = "VeryLazy" },
  { "RRethy/vim-illuminate",                       event = "VeryLazy" },
  { "lewis6991/gitsigns.nvim",                     event = "BufRead" },
  { "j-hui/fidget.nvim",                           event = "VeryLazy" },
  { "karb94/neoscroll.nvim",                       event = "VeryLazy" },
  { "mhogeveen/reach.nvim",                        event = "VeryLazy" },
  { "famiu/bufdelete.nvim",                        event = "VeryLazy" },
  {
    "folke/todo-comments.nvim",
    event = "VeryLazy",
    dependencies = {
      "nvim-lua/plenary.nvim",
    },
  },
  { "melkster/modicator.nvim", event = "VeryLazy" },
  {
    "iamcco/markdown-preview.nvim",
    build = function()
      vim.fn["mkdp#util#install"]()
    end,
    ft = "markdown",
  },
}, {
  lockfile = "~/.dotfiles/neovim/.config/nvim/lazy-lock.json",
})
