return {
  --- https://github.com/jiaoshijie/undotree
  'jiaoshijie/undotree',
  dependencies = 'nvim-lua/plenary.nvim',
  config = true,
  keys = {
    { '<leader>u', "<CMD>lua require('undotree').toggle()<CR>" },
  },
}
