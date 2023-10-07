return {
  {
    'catppuccin/nvim',
    name = 'catppuccin',
    lazy = false,
    priority = 1000,
    config = function()
      -- Set Catppuccin flavour (can be set with the :Catppuccin <flavour> cmd on the fly)
      vim.g.catppuccin_flavour = 'mocha' -- latte, frappe, macchiato, mocha

      -- Run theme setup function
      require('catppuccin').setup {
        transparent_background = false,
        styles = {
          comments = { 'italic' },
          conditionals = {},
          loops = {},
          functions = { 'italic' },
          keywords = {},
          strings = {},
          variables = {},
          numbers = {},
          booleans = {},
          properties = {},
          types = {},
          operators = {},
        },
        integrations = {
          cmp = true,
          treesitter = true,
          telescope = true,
          fidget = true,
          gitsigns = true,
        },
        highlight_overrides = {
          mocha = function(colors)
            return {
              FidgetTitle = { fg = colors.blue, bg = colors.none },
              FidgetTask = { fg = colors.blue, bg = colors.none },
              NormalFloat = { bg = colors.none },
            }
          end,
        },
      }

      -- Set colorscheme with Vim cmd
      vim.cmd 'colorscheme catppuccin'
    end,
  },
  {
    'melkster/modicator.nvim',
    event = 'VeryLazy',
    config = function()
      local mocha = require('catppuccin.palettes').get_palette 'mocha'

      require('modicator').setup {
        show_warnings = false, -- Show warning if any required option is missing
        highlights = {
          -- Default options for bold/italic
          defaults = {
            bold = false,
            italic = false,
          },
        },
      }

      local mode_color_map = {
        ['NormalMode'] = mocha.blue,
        ['InsertMode'] = mocha.green,
        ['VisualMode'] = mocha.mauve,
        ['CommandMode'] = mocha.peach,
        ['ReplaceMode'] = mocha.red,
        ['SelectMode'] = mocha.mauve,
      }

      for mode, color in pairs(mode_color_map) do
        vim.api.nvim_set_hl(0, mode, {
          fg = color,
        })
      end
    end,
  },
}
