local status_modicator, modicator = pcall(require, "modicator")
if not status_modicator then
  print("Modicator is not installed")
  return
end

local status_catppuccin, catppuccin = pcall(require, "catppuccin.palettes")
if not status_catppuccin then
  print("Catppuccin is not installed")
  return
end

local mocha = catppuccin.get_palette("mocha")

modicator.setup({
  show_warnings = true, -- Show warning if any required option is missing
  highlights = {
    -- Default options for bold/italic
    defaults = {
      bold = false,
      italic = false,
    },
  },
})

local mode_color_map = {
  ["NormalMode"] = mocha.blue,
  ["InsertMode"] = mocha.green,
  ["VisualMode"] = mocha.mauve,
  ["CommandMode"] = mocha.peach,
  ["ReplaceMode"] = mocha.red,
  ["SelectMode"] = mocha.mauve,
}

for mode, color in pairs(mode_color_map) do
  vim.api.nvim_set_hl(0, mode, {
    fg = color,
  })
end
