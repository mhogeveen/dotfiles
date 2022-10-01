local status, wezterm = pcall(require, "wezterm")
if not status then
  return
end

return {
  automatically_reload_config = true,
  font = wezterm.font("PragmataPro Mono Liga"),
  font_size = 15.0,
  line_height = 1.4,
  color_scheme = "Catppuccin Mocha",
  window_padding = { left = 0, right = 0, top = 0, bottom = 0 },
  window_decorations = "NONE | RESIZE",

  enable_tab_bar = true,
  use_fancy_tab_bar = false,
  tab_bar_at_bottom = true,
  show_tab_index_in_tab_bar = true,
  hide_tab_bar_if_only_one_tab = true,
}
