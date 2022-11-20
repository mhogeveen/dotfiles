local status, fidget = pcall(require, "fidget")
if not status then
	print("Fidget is not installed")
	return
end

fidget.setup({
	text = {
		spinner = "dots_pulse",
		done = " ",
	},
	window = {
		blend = 0,
	},
})
