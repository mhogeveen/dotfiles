---@type LazySpec
return {
  --- https://github.com/j-hui/fidget.nvim
  'j-hui/fidget.nvim',
  event = 'LspAttach',
  opts = {
    notification = {
      window = {
        winblend = 0,
      },
    },
  },
}
