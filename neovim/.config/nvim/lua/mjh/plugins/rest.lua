---@type LazySpec
return {
  --- https://github.com/mistweaverco/kulala.nvim
  'mistweaverco/kulala.nvim',
  enabled = true,
  ft = { 'http', 'rest' },
  keys = {
    {
      '<leader>ko',
      function()
        require('kulala').open()
      end,
      mode = 'n',
      ft = { 'http', 'rest' },
      desc = 'kulala - open',
    },
    {
      '<leader>kr',
      function()
        require('kulala').run()
      end,
      mode = 'n',
      ft = { 'http', 'rest' },
      desc = 'kulala - run',
    },
    {
      '<leader>ka',
      function()
        require('kulala').run_all()
      end,
      mode = 'n',
      ft = { 'http', 'rest' },
      desc = 'kulala - run all',
    },
  },
  opts = {
    default_view = 'headers_body',
  },
}
