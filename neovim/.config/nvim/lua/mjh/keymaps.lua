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

-- Move buffers around
map('n', '<leader>mh', '<C-w>H')
map('n', '<leader>mj', '<C-w>J')
map('n', '<leader>mk', '<C-w>K')
map('n', '<leader>ml', '<C-w>L')

-- Vertical movement
-- map('n', '<C-d>', '<C-d>zz') -- doesn't work well with snacks.nvim scroll
-- map('n', '<C-u>', '<C-u>zz') -- doesn't work well with snacks.nvim scroll

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

-- Folds
-- Defaults:
-- zf#j creates a fold from the cursor down # lines.
-- zf/string creates a fold from the cursor to string .
-- zj moves the cursor to the next fold.
-- zk moves the cursor to the previous fold.
-- zo opens a fold at the cursor.
-- zO opens all folds at the cursor.
-- zm increases the foldlevel by one.
-- zc closes a fold at the cursor.
-- zM closes all open folds.
-- zr decreases the foldlevel by one.
-- zR decreases the foldlevel to zero -- all folds will be open.
-- zd deletes the fold at the cursor.
-- zE deletes all folds.
-- [z move to start of open fold.
-- ]z move to end of open fold.
map('n', ']f', 'zj')
map('n', '[f', 'zk')
map('n', 'fo', 'zo')
map('n', 'fO', 'zO')
map('n', 'fa', 'zO')
map('n', 'fc', 'zc')
map('n', 'fC', 'zM')
map('n', 'fs', '[z')
map('n', 'fe', ']z')
