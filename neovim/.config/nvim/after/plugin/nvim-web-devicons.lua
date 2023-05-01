local ok, devicons = pcall(require, "nvim-web-devicons")
if not ok then
  print("'nnvim-web-devicons' is not installed")
  return
end

devicons.setup({
  override_by_filename = {
    ["containerfile"] = {
      icon = "",
      color = "#458ee6",
      cterm_color = "68",
      name = "Dockerfile",
    },
    ["docker-compose.yml"] = {
      icon = "",
      color = "#458ee6",
      cterm_color = "68",
      name = "Dockerfile",
    },
    ["docker-compose.yaml"] = {
      icon = "",
      color = "#458ee6",
      cterm_color = "68",
      name = "Dockerfile",
    },
    [".dockerignore"] = {
      icon = "",
      color = "#458ee6",
      cterm_color = "68",
      name = "Dockerfile",
    },
  },
})
