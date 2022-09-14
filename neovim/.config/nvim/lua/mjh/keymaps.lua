local map = vim.keymap.set
local opts = { noremap = true, silent = true }

--Remap space as leader key
map("", "<Space>", "<Nop>", opts)
vim.g.mapleader = " "
vim.g.maplocalleader = " "

-- Normal Mode --
-- Better window navigation
map("n", "<C-h>", "<C-w>h", opts)
map("n", "<C-j>", "<C-w>j", opts)
map("n", "<C-k>", "<C-w>k", opts)
map("n", "<C-l>", "<C-w>l", opts)

-- Vertical movement
map("n", "<C-d>", "<C-d>zz")
map("n", "<C-u>", "<C-u>zz")

-- Resize with arrows
map("n", "<A-Up>", ":resize +2<CR>", opts)
map("n", "<A-Down>", ":resize -2<CR>", opts)
map("n", "<A-Left>", ":vertical resize -2<CR>", opts)
map("n", "<A-Right>", ":vertical resize +2<CR>", opts)

-- Move lines up and down
map("n", "<C-p>", ":m-2<CR>==", opts)
map("n", "<C-n>", ":m+<CR>==", opts)

-- Delete without overwriting yank register
map("n", "dd", '"_dd', opts)

-- NvimTree
map("n", "<leader>e", ":NvimTreeToggle<cr>", opts)

-- Neogit
map("n", "<leader>t", ":Neogit<cr>", opts)

-- Telescope
map("n", "<leader>ff", ":Telescope find_files<CR>", opts)
map("n", "<leader>fg", ":Telescope git_files<CR>", opts)
map("n", "<leader>fb", ":Telescope buffers<CR>", opts)
map("n", "<leader>fr", ":Telescope live_grep<CR>", opts)

-- Barbar
map("n", "<space>,", ":BufferPrevious<CR>", opts)
map("n", "<space>.", ":BufferNext<CR>", opts)
map("n", "<space>c", ":BufferClose<CR>", opts)
map("n", "<space>p", ":BufferPick<CR>", opts)
map("n", "<space>bb", ":BufferOrderByBufferNumber<CR>", opts)
map("n", "<space>bd", ":BufferOrderByDirectory<CR>", opts)
map("n", "<space>bl", ":BufferOrderByLanguage<CR>", opts)
map("n", "<space>bw", ":BufferOrderByWindowNumber<CR>", opts)

-- Visual Mode --
-- Move lines up and down
map("v", "<C-p>", ":m '<-2<CR>gv", opts)
map("v", "<C-n>", ":m '>+1<CR>gv=gv", opts)

-- Stay in indent mode
map("v", "<", "<gv", opts)
map("v", ">", ">gv", opts)
