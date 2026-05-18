---@type LazySpec
return {
  --- https://github.com/ya2s/nvim-nonicons
  'yamatsum/nvim-nonicons',
  lazy = true,
  dependencies = {
    {
      --- https://github.com/nvim-tree/nvim-web-devicons
      'nvim-tree/nvim-web-devicons',
      lazy = true,
    },
  },
  opts = {},
}
