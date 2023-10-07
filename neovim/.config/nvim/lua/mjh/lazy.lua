-- Bootstrap Lazy.nvim
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

-- Setup plugins
--
-- All modules files in `/lua/mjh/plugins/*.lua` will be merged
-- automatically into the plugin spec.
require("lazy").setup("mjh.plugins", {
  lockfile = "~/.dotfiles/neovim/.config/nvim/lazy-lock.json",
})
