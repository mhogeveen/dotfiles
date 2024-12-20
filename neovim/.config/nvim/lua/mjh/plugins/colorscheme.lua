---@type LazySpec
return {
  -- https://github.com/catppuccin/nvim
  'catppuccin/nvim',
  name = 'catppuccin',
  lazy = true,
  priority = 1000,
  init = function()
    -- Set Catppuccin flavour (can be set with the :Catppuccin <flavour> cmd on the fly)
    vim.g.catppuccin_flavour = 'mocha' -- latte, frappe, macchiato, mocha
    -- Set colorscheme with Vim cmd
    vim.cmd 'colorscheme catppuccin'
  end,
  opts = {
    transparent_background = false,
    styles = {
      comments = { 'italic' },
      conditionals = {},
      loops = {},
      functions = { 'italic' },
      keywords = { 'italic' },
      strings = {},
      variables = {},
      numbers = {},
      booleans = {},
      properties = {},
      types = { 'italic' },
      operators = {},
    },
    integrations = {
      alpha = true,
      blink_cmp = true,
      cmp = false,
      fidget = false,
      gitsigns = true,
      native_lsp = {
        enabled = true,
        virtual_text = {
          errors = { 'italic' },
          hints = { 'italic' },
          warnings = { 'italic' },
          information = { 'italic' },
        },
        underlines = {
          errors = { 'underline' },
          hints = { 'underline' },
          warnings = { 'underline' },
          information = { 'underline' },
        },
        inlay_hints = {
          background = true,
        },
      },
      fzf = true,
      treesitter = true,
    },
    highlight_overrides = {
      mocha = function(mocha)
        return {
          NormalFloat = { bg = mocha.none },
        }
      end,
    },
  },
}
