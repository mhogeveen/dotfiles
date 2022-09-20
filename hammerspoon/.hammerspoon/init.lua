require("settings")
local quake = require("quake")
local geo = require("geo")

hs.loadSpoon("ControlEscape"):start()

-- Set up quake apps
quake.init({ "alt" }, "space", "kitty", geo.full)
quake.init({ "alt", "shift" }, "space", "kitty", geo.terminalBottom)
quake.init({ "alt" }, "s", "Spotify", geo.centered)

-- Reload Hammerspoon config
hs.hotkey.bind({ "cmd", "alt", "ctrl" }, "r", function()
	hs.reload()
	hs.alert.closeAll()
	hs.alert.show("🔧 Config reloaded")
end)

hs.application.enableSpotlightForNameSearches(true)
