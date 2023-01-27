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
    modes = {
      ["n"] = mocha.blue, -- Normal
      ["i"] = mocha.green, -- Insert
      ["v"] = mocha.mauve, -- Visual
      ["V"] = mocha.mauve, -- Visual Line
      [""] = mocha.mauve, -- Visual Block
      ["R"] = mocha.red, -- Replace
      ["c"] = mocha.peach, -- Command
    },
  },
})
