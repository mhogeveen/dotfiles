---@type LazySpec
return {
  --- https://github.com/folke/snacks.nvim
  'folke/snacks.nvim',
  priority = 1000,
  lazy = false,
  keys = {
    {
      '<leader>c',
      function()
        Snacks.bufdelete.delete()
      end,
      mode = 'n',
      desc = 'Delete current buffer',
      noremap = true,
      silent = true,
    },
    {
      '<leader>a',
      function()
        Snacks.bufdelete.all()
      end,
      mode = 'n',
      desc = 'Delete all buffers',
      noremap = true,
      silent = true,
    },
    {
      '<loader>o',
      function()
        Snacks.bufdelete.other()
      end,
      mode = 'n',
      desc = 'Delete other buffers',
      noremap = true,
      silent = true,
    },
    {
      '<leader>lg',
      function()
        Snacks.terminal('lazygit -ucf ~/.config/lazygit/config.yml', {
          auto_close = true,
          win = {
            border = 'single',
            title = ' Lazygit - CWD ',
          },
        })
      end,
      mode = 'n',
      desc = 'Floating terminal with Lazygit',
      noremap = true,
      silent = true,
    },
    {
      '<leader>og',
      function()
        local Obsidian = require 'mjh.utils.obsidian'
        Snacks.terminal('lazygit -ucf ~/.config/lazygit/config.yml', {
          auto_close = true,
          cwd = Obsidian.resolve_obsidian_vault(Obsidian.VaultType.Main).absolute_path,
          win = {
            border = 'single',
            title = ' Lazygit - Obsidian ',
          },
        })
      end,
      mode = 'n',
      desc = 'Floating terminal with Lazygit for Obsidian vault',
      noremap = true,
      silent = true,
    },
  },
  ---@type snacks.Config
  opts = {
    bigfile = { enabled = true },
    bufdelete = { enabled = true },
    scroll = {
      enabled = true,
      animate_repeat = {
        delay = 0,
      },
    },
    terminal = { enabled = true },
  },
}
