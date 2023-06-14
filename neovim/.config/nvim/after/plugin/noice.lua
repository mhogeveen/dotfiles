local status, noice = pcall(require, "noice")
if not status then
  print("'noice' is not installed")
  return
end

noice.setup({
})
