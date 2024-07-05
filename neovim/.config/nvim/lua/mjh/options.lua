local options = {
  number = true, -- precede each line with its line number
  cursorline = true, -- highlight the text line of the cursor
  signcolumn = 'yes:1', -- always display signcolumn of fixed width 1
  timeoutlen = 500, -- time in ms to wait for a mapped sequence to complete
  updatetime = 250, -- cursorhold time in ms
  mouse = 'a', -- enable mouse in all modes
  ignorecase = true, -- ignore case in search patterns
  smartcase = true, -- makes ignore case smarter in certain circumstances
  wrapscan = true, -- searches wrap around the end of the file
  swapfile = false, -- disable swapfiles
  inccommand = 'split', -- ?? recommended everywhere but not sure what it does (:h not helping)
  hlsearch = false, -- highlight previous search pattern matches
  breakindent = true, -- continue visual indentation of wrapped lines
  wrap = false, -- do not wrap lines by defualt
  clipboard = 'unnamedplus', -- copying goes into OS clipboard as well
  pumheight = 10, -- max number of items in pop-up menu
  splitbelow = true, -- vertical split is created below current buffer
  splitright = true, -- hoizontal split is created to the right of current buffer
  scrolloff = 8, -- scroll offset from top and bottom of file
  sidescrolloff = 8, -- scroll offset from left and right of file
  undofile = true, -- save undo history to a file
  shiftwidth = 2, -- number of spaces to use for each step of (auto)indent
}

for k, v in pairs(options) do
  vim.opt[k] = v
end
