local opts = { noremap = true, silent = true }
local map = vim.keymap.set

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
map("n", "<leader>fr", ":Telescope live_grep<CR>", opts)

-- Buffer stuff
-- map("n", "<leader>b", ":Telescope buffers<CR>", opts)
map("n", "<leader>c", ":Bdelete<CR>", opts)

-- Visual Mode --
-- Move lines up and down
map("v", "<C-p>", ":m '<-2<CR>gv", opts)
map("v", "<C-n>", ":m '>+1<CR>gv=gv", opts)

-- Stay in indent mode
map("v", "<", "<gv", opts)
map("v", ">", ">gv", opts)

-- Insert Mode --
-- Luasnip
-- map({ "i", "s" }, "<Tab>", function()
-- 	local ls = require("luasnip")
-- 	if ls.expand_or_jumpable() then
-- 		ls.expand_or_jump()
-- 	end
-- end, { silent = true })
