local status, telescope = pcall(require, "telescope")
if not status then
	return
end

telescope.setup({
	pickers = {
		buffers = {
			theme = "dropdown",
			previewer = false,
		},
	},
})

telescope.load_extension("fzf")
