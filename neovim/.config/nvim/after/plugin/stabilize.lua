local status, stabilize = pcall(require, "stabilize")
if not status then
	return
end

stabilize.setup()
