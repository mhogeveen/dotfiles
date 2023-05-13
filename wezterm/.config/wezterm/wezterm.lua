local status, wezterm = pcall(require, "wezterm")
if not status then
  return
end

local act = wezterm.action
local config = {}

-- In newer versions of wezterm, use the config_builder which will
-- help provide clearer error messages
if wezterm.config_builder then
  config = wezterm.config_builder()
end

-- General
config.automatically_reload_config = true

-- Font
config.font = wezterm.font("BlexMono Nerd Font Mono")
config.font_size = 13.5
config.line_height = 1.2

-- Colors
config.color_scheme = "Catppuccin Mocha"

-- Window
config.window_padding = { left = 10, right = 10, top = 10, bottom = 10 }
config.window_decorations = "NONE | RESIZE"

-- Tabbar
config.enable_tab_bar = true
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = true
config.show_tab_index_in_tab_bar = true
config.hide_tab_bar_if_only_one_tab = true

-- Keybindings
config.leader = { key = "a", mods = "CTRL", timeout_milliseconds = 500 }
config.keys = {
  {
    key = "\\",
    mods = "LEADER",
    action = act.SplitHorizontal({ domain = "CurrentPaneDomain" }),
  },
  {
    key = "-",
    mods = "LEADER",
    action = act.SplitVertical({ domain = "CurrentPaneDomain" }),
  },
  {
    key = "r",
    mods = "LEADER",
    action = act.ActivateKeyTable({
      name = "resize_pane",
      one_shot = false,
    }),
  },
  { key = "h", mods = "LEADER", action = act.ActivatePaneDirection("Left") },
  { key = "l", mods = "LEADER", action = act.ActivatePaneDirection("Right") },
  { key = "k", mods = "LEADER", action = act.ActivatePaneDirection("Up") },
  { key = "j", mods = "LEADER", action = act.ActivatePaneDirection("Down") },
  { key = 'p', mods = 'LEADER', action = act.ActivateTabRelative(-1) },
  { key = 'n', mods = 'LEADER', action = act.ActivateTabRelative(1) },
  { key = '[', mods = 'LEADER', action = act.MoveTabRelative(-1) },
  { key = ']', mods = 'LEADER', action = act.MoveTabRelative(1) },
}

for i = 0, 9 do
  table.insert(config.keys, {
    key = tostring(i),
    mods = 'LEADER',
    action = act.ActivateTab(i - 1),
  })
end

config.key_tables = {
  resize_pane = {
    { key = "LeftArrow",  action = act.AdjustPaneSize({ "Left", 2 }) },
    { key = "h",          action = act.AdjustPaneSize({ "Left", 2 }) },
    { key = "RightArrow", action = act.AdjustPaneSize({ "Right", 2 }) },
    { key = "l",          action = act.AdjustPaneSize({ "Right", 2 }) },
    { key = "UpArrow",    action = act.AdjustPaneSize({ "Up", 2 }) },
    { key = "k",          action = act.AdjustPaneSize({ "Up", 2 }) },
    { key = "DownArrow",  action = act.AdjustPaneSize({ "Down", 2 }) },
    { key = "j",          action = act.AdjustPaneSize({ "Down", 2 }) },
    { key = "Escape",     action = "PopKeyTable" },
  },
}

return config
