local status_alpha, alpha = pcall(require, "alpha")
if not status_alpha then
  print("Alpha is not installed")
  return
end

local theme = require("alpha.themes.dashboard")
local ascii = require("mjh.utils.ascii")

theme.section.header.val = ascii.relief

theme.section.buttons.val = {
  theme.button("f", "  - Find file", ":Telescope find_files<CR>"),
  theme.button("r", "  - Recent", ":Telescope oldfiles<CR>"),
  theme.button("n", "  - Nvim Config", ":cd ~/.dotfiles/neovim/.config/nvim | e .<CR>"),
  theme.button("t", "  - Kitty Config", ":cd ~/.dotfiles/kitty/.config/kitty | e .<CR>"),
  theme.button("q", "  - Quit", ":qa<CR>"),
}
theme.section.buttons.opts.spacing = 0

theme.section.footer.val = require("alpha.fortune")

theme.config.layout = {
  { type = "padding", val = 4 },
  theme.section.header,
  { type = "padding", val = 4 },
  theme.section.buttons,
  { type = "padding", val = 4 },
  theme.section.footer,
}

alpha.setup(theme.opts)

vim.cmd([[ autocmd FileType alpha setlocal nofoldenable ]])
