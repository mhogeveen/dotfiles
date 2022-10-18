local status, modes = pcall(require, "modes")
if not status then
	print("Modes is not installed")
	return
end

modes.setup({
	colors = {
		copy = "#f9e2af",
		delete = "#f38ba8",
		insert = "#a6e3a1",
		visual = "#cba6f7",
	},

	-- Set opacity for cursorline and number background
	line_opacity = 0.1,

	-- Enable cursor highlights
	set_cursor = true,

	-- Enable cursorline initially, and disable cursorline for inactive windows
	-- or ignored filetypes
	set_cursorline = true,

	-- Enable line number highlights to match cursorline
	set_number = true,

	-- Disable modes highlights in specified filetypes
	-- Please PR commonly ignored filetypes
	ignore_filetypes = { "NvimTree", "TelescopePrompt" },
})
