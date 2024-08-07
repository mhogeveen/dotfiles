---@param mode string | string[]
---@param keymap string
---@param command string | function
local map = function(mode, keymap, command)
  vim.keymap.set(mode, keymap, command, {
    noremap = true,
    silent = true,
  })
end

--Remap space as leader key
map('', '<Space>', '<Nop>')
vim.g.mapleader = ' '
vim.g.maplocalleader = ' '

-- Normal Mode --
-- Better window navigation
map('n', '<C-h>', '<C-w>h')
map('n', '<C-j>', '<C-w>j')
map('n', '<C-k>', '<C-w>k')
map('n', '<C-l>', '<C-w>l')

-- Vertical movement
map('n', '<C-d>', '<C-d>zz')
map('n', '<C-u>', '<C-u>zz')

-- Resize with arrows
map('n', '<A-Up>', ':resize +2<CR>')
map('n', '<A-Down>', ':resize -2<CR>')
map('n', '<A-Left>', ':vertical resize -2<CR>')
map('n', '<A-Right>', ':vertical resize +2<CR>')

-- Move lines up and down
map('n', '<C-p>', ':m-2<CR>==')
map('n', '<C-n>', ':m+<CR>==')

-- Delete without overwriting yank register
map('n', 'dd', '"_dd')

-- Visual Mode --
-- Move lines up and down
map('v', '<C-p>', ":m '<-2<CR>gv")
map('v', '<C-n>', ":m '>+1<CR>gv=gv")

-- Stay in indent mode
map('v', '<', '<gv')
map('v', '>', '>gv')

-- Sort selection
map('v', '<C-s>', ":'<,'>sort<CR>")
