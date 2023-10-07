return {
  'j-hui/fidget.nvim',
  event = 'VeryLazy',
  tag = 'legacy',
  config = function()
    require('fidget').setup {
      text = {
        spinner = 'dots_pulse',
        done = ' ',
      },
      window = {
        blend = 0,
      },
    }
  end,
}
