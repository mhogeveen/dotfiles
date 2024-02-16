local wezterm = require 'wezterm'

wezterm.on('user-var-changed', function(window, _, name, value)
  local overrides = window:get_config_overrides() or {}

  if name == 'neovim' then
    if value == 'true' then
      overrides.window_padding = { left = 0, right = 0, top = 0, bottom = 0 }
    else
      overrides = {}
    end
  end

  window:set_config_overrides(overrides)
end)
