local status, stabilize = pcall(require, "stabilize")
if not status then
  print("Stabilize is not installed")
  return
end

stabilize.setup()
