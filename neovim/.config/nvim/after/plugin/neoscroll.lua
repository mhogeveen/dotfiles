local status, neoscroll = pcall(require, "neoscroll")
if not status then
  print("Neoscroll is not installed")
  return
end

neoscroll.setup()
