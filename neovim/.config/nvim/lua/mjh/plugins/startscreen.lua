return {
  --- https://github.com/goolord/alpha-nvim
  'goolord/alpha-nvim',
  dependencies = { 'nvim-tree/nvim-web-devicons' },
  opts = function()
    local theme = require 'alpha.themes.dashboard'
    local ascii = require 'mjh.utils.ascii'

    theme.section.header.val = ascii.relief

    theme.section.buttons.val = {
      theme.button('f', '  - Find file', ':Telescope find_files<CR>'),
      theme.button('r', '  - Recent', ':Telescope oldfiles<CR>'),
      theme.button('n', '  - Nvim Config', ':cd ~/.dotfiles/neovim/.config/nvim | e .<CR>'),
      theme.button('q', '  - Quit', ':qa<CR>'),
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
