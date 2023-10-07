local module = {}

local spaces = require 'hs.spaces'

-- Bind application as 'quake' app
module.init = function(mods, key, appName, rect)
  hs.hotkey.bind(mods, key, function()
    local app = hs.application.get(appName)

    if not app or app:isHidden() then
      hs.application.launchOrFocus(appName)
      app = hs.application.get(appName)
    elseif hs.application.frontmostApplication() ~= app then
      app:activate()
    else
      app:hide()
    end

    if app then
      app:mainWindow():moveToUnit(rect)
    end
  end)
end

return module
