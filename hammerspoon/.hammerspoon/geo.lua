local module = {}

-- Geometry rect (x, y, w, h)
--    x: x coordinate from top left (0 - 1)
--    y: y coordinate from top left (0 - 1)
--    w: width (0 - 1)
--    h: height (0 - 1)

-- Full screen
module.full = hs.geometry.rect(0, 0, 1, 1)

-- Bottom 60% of the screen
module.terminalBottom = hs.geometry.rect(0, 0.4, 1, 0.6)

-- Centered
module.centered = hs.geometry.rect(0.25, 0.15, 0.5, 0.7)

return module
