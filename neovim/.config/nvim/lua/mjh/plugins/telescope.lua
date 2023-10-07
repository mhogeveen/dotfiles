return {
  'nvim-telescope/telescope.nvim',
  branch = '0.1.x',
  cmd = 'Telescope',
  dependencies = {
    { 'nvim-telescope/telescope-fzf-native.nvim', build = 'make' },
    { 'nvim-telescope/telescope-ui-select.nvim' },
    { 'nvim-lua/plenary.nvim' },
  },
  config = function()
    local telescope = require 'telescope'

    telescope.setup {
      defaults = {
        borderchars = { '─', '│', '─', '│', '┌', '┐', '┘', '└' },
        mappings = {
          n = {
            ['<C-d>'] = require('telescope.actions').delete_buffer,
          },
          i = {
            ['<C-d>'] = require('telescope.actions').delete_buffer,
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
          require('telescope.themes').get_dropdown {
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

    pcall(function()
      telescope.load_extension 'fzf'
    end)
    pcall(function()
      telescope.load_extension 'ui-select'
    end)
  end,
}