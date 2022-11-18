local status, fidget = pcall(require, "fidget")
if not status then
  print("Fidget is not installed")
  return
end

fidget.setup({
  window = {
    blend = 0,
  },
})
