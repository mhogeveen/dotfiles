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
      ["n"] = {
        foreground = mocha.blue, -- Normal
      },
      ["i"] = {
        foreground = mocha.green, -- Insert
      },
      ["v"] = {
        foreground = mocha.mauve, -- Visual
      },
      ["V"] = {
        foreground = mocha.mauve, -- Visual Line
      },
      [""] = {
        foreground = mocha.mauve, -- Visual Block
      },
      ["R"] = {
        foreground = mocha.red, -- Replace
      },
      ["c"] = {
        foreground = mocha.peach, -- Command
      },
    },
  },
})
