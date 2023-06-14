local wezterm = require("wezterm")

-- Color scheme reference:
-- https://github.com/catppuccin/wezterm/blob/main/plugin/init.lua
local scheme = wezterm.get_builtin_color_schemes()["Catppuccin Mocha"]
local black = scheme.tab_bar.background
local white = scheme.foreground
local blue = scheme.ansi[5]

local function tab_title(tab_info)
  local title = tab_info.tab_title
  if title and #title > 0 then
    return title
  end
  return tab_info.active_pane.title
end

wezterm.on("format-tab-title", function(tab, tabs, panes, config, hover, max_width)
  local is_last_tab = #tabs == (tab.tab_index + 1)
  local divider = is_last_tab and " " or " | "

  local title = tab.tab_index .. ":" .. tab_title(tab)
  title = wezterm.truncate_right(title, max_width - #divider - 2)
  title = wezterm.pad_right(title, (max_width - #divider) - 1)

  return {
    { Foreground = { Color = tab.is_active and black or white } },
    { Background = { Color = tab.is_active and blue or black } },
    { Text = " " .. title },
    { Foreground = { Color = white } },
    { Background = { Color = black } },
    { Text = divider },
  }
end)
