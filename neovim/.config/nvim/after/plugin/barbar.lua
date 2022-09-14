local status, bufferline = pcall(require, "bufferline")
if not status then
  return
end

bufferline.setup({
  auto_hide = true,
  closable = false,
  insert_at_end = true,
  maximum_padding = 1,
})
