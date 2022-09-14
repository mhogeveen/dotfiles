local status, telescope = pcall(require, "telescope")
if not status then
  return
end

telescope.setup({
  pickers = {
    find_files = {
      theme = "dropdown",
      previewer = false,
    },
    buffers = {
      theme = "dropdown",
      previewer = false,
    },
  },
})
