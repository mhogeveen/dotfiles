local status, reach = pcall(require, "reach")
if not status then
  print("Reach is not installed")
  return
end

reach.setup({
  notifications = true,
})

local options = {
  handle = "auto",       -- 'bufnr' or 'dynamic' or 'auto'
  show_icons = true,
  show_current = true,   -- Include current buffer in the list
  show_modified = false, -- Show buffer modified indicator
  modified_icon = "⬤", -- Character to use as modified indicator
  grayout_current = true, -- Wheter to gray out current buffer entry
  force_delete = {},     -- List of filetypes / buftypes to use
  -- 'bdelete!' on, e.g. { 'terminal' }
  filter = nil,          -- Function taking bufnr as parameter,
  -- returning true or false
  sort = nil,            -- Comparator function (bufnr, bufnr) -> bool
  terminal_char = "\\",  -- Character to use for terminal buffer handles
  -- when options.handle is 'dynamic'
  grayout = true,        -- Gray out non matching entries

  -- A list of characters to use as handles when options.handle == 'auto'
  auto_handles = require("reach.buffers.constant").auto_handles,
  auto_exclude_handles = {}, -- A list of characters not to use as handles when
  -- options.handle == 'auto', e.g. { '8', '9', 'j', 'k' }
  previous = {
    enable = false, -- Mark last used buffers with specified chars and colors
    depth = 2,     -- Maximum number of buffers to mark
    chars = { "•" }, -- Characters to use as markers,
    -- last one is used when depth > #chars
    groups = {     -- Highlight groups for markers,
      "String",    -- last one is used when depth > #groups
      "Comment",
    },
  },
  -- A map of action to key that should be used to invoke it
  actions = {
    split = "-",
    vertsplit = "|",
    tabsplit = "]",
    delete = "<Space>",
    priority = "=",
  },
  window = {
    border = "single",
    title = "",
    title_pos = "center",
  },
}

vim.api.nvim_create_user_command("ReachToggle", function()
  reach.buffers(options)
end, {})
