return {
  --- https://github.com/echasnovski/mini.bufremove
  'echasnovski/mini.bufremove',
  version = '*',
  keys = {
    {
      '<leader>c',
      function()
        require('mini.bufremove').delete()
      end,
      mode = 'n',
      desc = 'Remove buffer',
      noremap = true,
      silent = true,
    },
  },
  opts = {},
}
