---@type LazySpec
return {
  --- https://github.com/nvim-neotest/neotest
  'nvim-neotest/neotest',
  dependencies = {
    {
      --- https://github.com/nvim-neotest/nvim-nio
      'nvim-neotest/nvim-nio',
    },
    {
      --- https://github.com/nvim-lua/plenary.nvim/
      'nvim-lua/plenary.nvim',
    },
    {
      --- https://github.com/nvim-treesitter/nvim-treesitter
      'nvim-treesitter/nvim-treesitter',
    },
    -- Adapters
    {
      --- https://github.com/marilari88/neotest-vitest
      'marilari88/neotest-vitest',
    },
  },
  keys = {
    {
      'trc',
      ':lua require("neotest").run.run(vim.fn.expand("%"))<CR>',
      mode = 'n',
      desc = '(T)est (R)un (C)urrent',
      noremap = true,
      silent = true,
    },
    {
      'trl',
      ':lua require("neotest").run.run_last()<CR>',
      mode = 'n',
      desc = '(T)est (R)un (L)ast',
      noremap = true,
      silent = true,
    },
    {
      'tww',
      ':lua require("neotest").watch.watch(vim.fn.expand("%"))<CR>',
      mode = 'n',
      desc = '(T)est (W)atch (W)atch',
      noremap = true,
      silent = true,
    },
    {
      'tws',
      ':lua require("neotest").watch.stop()<CR>',
      mode = 'n',
      desc = '(T)est (W)atch (S)top',
      noremap = true,
      silent = true,
    },
    {
      'twt',
      ':lua require("neotest").watch.toggle(vim.fn.expand("%"))<CR>',
      mode = 'n',
      desc = '(T)est (W)atch (T)oggle',
      noremap = true,
      silent = true,
    },
    {
      'ts',
      ':lua require("neotest").summary.toggle()<CR>',
      mode = 'n',
      desc = '(T)est (S)ummary toggle',
      noremap = true,
      silent = true,
    },
    {
      'to',
      ':lua require("neotest").output_panel.toggle()<CR>',
      mode = 'n',
      desc = '(T)est (O)utput panel toggle',
      noremap = true,
      silent = true,
    },
    {
      't]',
      ':lua require("neotest").jump.next()<CR>',
      mode = 'n',
      desc = '(T)est jump next',
      noremap = true,
      silent = true,
    },
    {
      't[',
      ':lua require("neotest").jump.prev()<CR>',
      mode = 'n',
      desc = '(T)est jump prev',
      noremap = true,
      silent = true,
    },
  },
  opts = function()
    return {
      adapters = {
        require 'neotest-vitest',
      },
    }
  end,
}
