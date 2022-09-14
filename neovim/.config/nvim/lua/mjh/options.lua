local options = {
  number = true,
  relativenumber = true,
  cursorline = true,
  signcolumn = "yes",
  ttimeoutlen = 10,
  timeoutlen = 500,
  updatetime = 50,
  numberwidth = 4,
  mouse = "a",
  title = true,
  autoindent = true,
  smartindent = true,
  incsearch = true,
  ignorecase = true,
  smartcase = true,
  wrapscan = true,
  backup = false,
  swapfile = false,
  showcmd = false,
  cmdheight = 1,
  hlsearch = false,
  laststatus = 2,
  shell = "zsh",
  smarttab = true,
  breakindent = true,
  shiftwidth = 2,
  tabstop = 2,
  softtabstop = 2,
  expandtab = true,
  wrap = false,
  termguicolors = true,
  clipboard = "",
  conceallevel = 0,
  pumheight = 10,
  showmode = false,
  splitbelow = true,
  splitright = true,
  scrolloff = 8,
  sidescrolloff = 8,
  whichwrap = "b,s,<,>,[,]",
}

vim.opt.shortmess:append("c")

for k, v in pairs(options) do
  vim.opt[k] = v
end
