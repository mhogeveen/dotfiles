---@type LazySpec
return {
  --- https://github.com/folke/snacks.nvim
  'folke/snacks.nvim',
  priority = 1000,
  lazy = false,
  keys = {
    {
      'dc',
      function()
        Snacks.bufdelete.delete()
      end,
      mode = 'n',
      desc = 'Delete current buffer',
      noremap = true,
      silent = true,
    },
    {
      'da',
      function()
        Snacks.bufdelete.all()
      end,
      mode = 'n',
      desc = 'Delete all buffers',
      noremap = true,
      silent = true,
    },
    {
      'do',
      function()
        Snacks.bufdelete.other()
      end,
      mode = 'n',
      desc = 'Delete other buffers',
      noremap = true,
      silent = true,
    },
    {
      'lg',
      function()
        Snacks.terminal 'lazygit -ucf ~/.config/lazygit/config.yml'
      end,
      mode = 'n',
      desc = 'Floating terminal with Lazygit',
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
