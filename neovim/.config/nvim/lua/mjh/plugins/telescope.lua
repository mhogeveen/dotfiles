return {
  --- https://github.com/nvim-telescope/telescope.nvim
  'nvim-telescope/telescope.nvim',
  branch = '0.1.x',
  keys = {
    {
      '<leader>ff',
      ':Telescope find_files<CR>',
      mode = 'n',
      desc = 'Telescope - find files',
      noremap = true,
      silent = true,
    },
    {
      '<leader>fg',
      ':Telescope git_files<CR>',
      mode = 'n',
      desc = 'Telescope - git files',
      noremap = true,
      silent = true,
    },
    {
      '<leader>fr',
      ':Telescope live_grep<CR>',
      mode = 'n',
      desc = 'Telescope - grep',
      noremap = true,
      silent = true,
    },
  },
  cmd = 'Telescope',
  event = 'LspAttach',
  dependencies = {
    {
      'nvim-telescope/telescope-fzf-native.nvim',
      build = 'make',
      enabled = vim.fn.executable 'make' == 1,
    },
    {
      'nvim-telescope/telescope-ui-select.nvim',
    },
    { 'nvim-lua/plenary.nvim' },
  },
  opts = function()
    local actions = require 'telescope.actions'
    local themes = require 'telescope.themes'

    return {
      defaults = {
        borderchars = { '─', '│', '─', '│', '┌', '┐', '┘', '└' },
        mappings = {
          n = {
            ['<C-d>'] = actions.delete_buffer,
          },
          i = {
            ['<C-d>'] = actions.delete_buffer,
          },
        },
      },
      pickers = {
        buffers = {
          theme = 'dropdown',
          previewer = false,
          initial_mode = 'normal',
        },
      },
      extensions = {
        ['fzf'] = {
          fuzzy = true, -- false will only do exact matching
          override_generic_sorter = true, -- override the generic sorter
          override_file_sorter = true, -- override the file sorter
          case_mode = 'smart_case', -- or "ignore_case" or "respect_case"
        },
        ['ui-select'] = {
          themes.get_dropdown {
            previewer = false,
            initial_mode = 'normal',
          },
        },
      },
      vimgrep_arguments = {
        'rg',
        '--color=never',
        '--no-heading',
        '--with-filename',
        '--line-number',
        '--column',
        '--smart-case',
        '--hidden',
        '--glob=!.git/',
      },
    }
  end,
  config = function(_, opts)
    local telescope = require 'telescope'
    telescope.setup(opts)
    telescope.load_extension 'fzf'
    telescope.load_extension 'ui-select'
  end,
}
