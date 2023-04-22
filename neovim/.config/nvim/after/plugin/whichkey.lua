local status, whichkey = pcall(require, "which-key")
if not status then
  print("Which-key is not installed")
  return
end

whichkey.setup({
  plugins = {
    marks = false,   -- shows a list of your marks on ' and `
    registers = false, -- shows your registers on " in NORMAL or <C-r> in INSERT mode
    spelling = {
      enabled = false, -- enabling this will show WhichKey when pressing z= to select spelling suggestions
    },
    presets = {
      operators = false, -- adds help for operators like d, y, ... and registers them for motion / text object completion
      motions = false,   -- adds help for motions
      text_objects = false, -- help for text objects triggered after entering an operator
      windows = false,   -- default bindings on <c-w>
      nav = false,       -- misc bindings to work with windows
      z = false,         -- bindings for folds, spelling and others prefixed with z
      g = false,         -- bindings for prefixed with g
    },
  },
  window = {
    border = "single", -- none, single, double, shadow
  },
})

whichkey.register({
  ["g"] = {
    name = "LSP",
    D = { "Go to declaration" },
    h = { "Show hover panel" },
    d = { "Go to definition" },
    t = { "Go to type definition" },
    i = { "Go to implementation" },
    s = { "Signature help" },
    S = { "Document symbols" },
    r = { "Show references" },
    R = { "Rename" },
    l = { "Show line diagnostics" },
    f = { "Format" },
    a = { "Code actions" },
  },
  -- Leader
  ["<leader>e"] = { "Toggle tree explorer" },
  ["<leader>t"] = { "Neogit" },
  ["<leader>b"] = { "Buffer list" },
  ["<leader>c"] = { "Close buffer" },
  ["<leader>f"] = {
    name = "Telescope",
    f = { "Find File" },
    g = { "Git files" },
    r = { "Live grep" },
    b = { "Buffers" },
  },
})
