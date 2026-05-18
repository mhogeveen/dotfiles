local options = {
  breakindent = true, -- continue visual indentation of wrapped lines
  clipboard = 'unnamedplus', -- copying goes into OS clipboard as well
  cursorline = true, -- highlight the text line of the cursor
  hlsearch = false, -- highlight previous search pattern matches
  ignorecase = true, -- ignore case in search patterns
  inccommand = 'split', -- ?? recommended everywhere but not sure what it does (:h not helping)
  list = true, -- enable list characters
  listchars = 'tab:»·,multispace:··· ', -- set list characters
  mouse = 'a', -- enable mouse in all modes
  number = true, -- precede each line with its line number
  pumheight = 10, -- max number of items in pop-up menu
  scrolloff = 8, -- scroll offset from top and bottom of file
  shiftwidth = 2, -- number of spaces to use for each step of (auto)indent
  sidescrolloff = 8, -- scroll offset from left and right of file
  signcolumn = 'yes:1', -- always display signcolumn of fixed width 1
  smartcase = true, -- makes ignore case smarter in certain circumstances
  splitbelow = true, -- vertical split is created below current buffer
  splitright = true, -- hoizontal split is created to the right of current buffer
  swapfile = false, -- disable swapfiles
  timeoutlen = 500, -- time in ms to wait for a mapped sequence to complete
  undofile = true, -- save undo history to a file
  updatetime = 250, -- cursorhold time in ms
  wrap = false, -- do not wrap lines by defualt
  wrapscan = true, -- searches wrap around the end of the file
}

-- Disable unused runtime plugins
vim.g.loaded_netrw = 1
vim.g.loaded_netrwPlugin = 1
vim.g.loaded_matchit = 1
vim.g.loaded_matchparen = 1
vim.g.loaded_gzip = 1
vim.g.loaded_tar = 1
vim.g.loaded_tarPlugin = 1
vim.g.loaded_zip = 1
vim.g.loaded_zipPlugin = 1

for k, v in pairs(options) do
  vim.opt[k] = v
end
