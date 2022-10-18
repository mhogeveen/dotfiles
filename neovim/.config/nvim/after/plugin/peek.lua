local status, peek = pcall(require, "peek")
if not status then
  print("Peek is not installed")
  return
end

peek.setup()

vim.api.nvim_create_user_command("PeekOpen", require("peek").open, {})
vim.api.nvim_create_user_command("PeekClose", require("peek").close, {})
