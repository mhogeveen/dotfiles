---@type LazySpec
return {
  --- https://github.com/goolord/alpha-nvim
  'goolord/alpha-nvim',
  opts = function()
    local theme = require 'alpha.themes.dashboard'
    local ascii = require 'mjh.utils.ascii'

    theme.section.header.val = ascii.relief

    theme.section.buttons.val = {
      theme.button('ef', 'Find file', ':FzfLua files<CR>'),
      theme.button('eg', 'Git files', ':FzfLua git_files<CR>'),
      theme.button('er', 'Live grep', ':FzfLua live_grep<CR>'),
      theme.button('d', 'Dotfiles', ':cd ~/.dotfiles | :Neotree focus right reveal<CR>'),
      theme.button('q', 'Quit', ':qa!<CR>'),
    }
    theme.section.buttons.opts.spacing = 0

    theme.section.footer.val = require 'alpha.fortune'

    theme.config.layout = {
      { type = 'padding', val = 4 },
      theme.section.header,
      { type = 'padding', val = 4 },
      theme.section.buttons,
      { type = 'padding', val = 4 },
      theme.section.footer,
    }

    return theme.opts
  end,
}
