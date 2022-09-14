local theme_name = "catppuccin" -- catppuccin, github-theme, kanagawa, rose-pine

local status, theme = pcall(require, theme_name)
if not status then
  return
end

local configs = {}

configs["github-theme"] = {
  theme_style = "dark_default",
  transparent = false,
  function_style = "italic",
  variable_style = "italic",
}

configs["kanagawa"] = {
  undercurl = true, -- enable undercurls
  commentStyle = { italic = true },
  functionStyle = { italic = true },
  keywordStyle = { italic = false },
  statementStyle = { bold = true },
  typeStyle = {},
  variablebuiltinStyle = { italic = true },
  specialReturn = true, -- special highlight for the return keyword
  specialException = true, -- special highlight for exception handling keywords
  transparent = false, -- do not set background color
  dimInactive = false, -- dim inactive window `:h hl-NormalNC`
  globalStatus = true, -- adjust window separators highlight for laststatus=3
  terminalColors = true, -- define vim.g.terminal_color_{0,17}
  colors = {},
  overrides = {},
  theme = "default",
}

configs["catppuccin"] = {
  dim_inactive = {
    enabled = true,
    shade = "dark",
    percentage = 0.15,
  },
  integrations = {
    barbar = true,
    cmp = true,
    hop = true,
    nvimtree = true,
    treesitter = true,
    telescope = true,
    which_key = true,
  },
}

configs["rose-pine"] = {
  dark_variant = "main", -- main, moon
}

-- Set Catppuccin flavour (can be set with the :Catppuccin <flavour> cmd on the fly)
if theme_name == "catppuccin" then
  vim.g.catppuccin_flavour = "mocha" -- latte, frappe, macchiato, mocha
end

-- Run theme setup function
theme.setup(configs[theme_name])

-- Set colorscheme with Vim cmd
if theme_name ~= "github-theme" then
  vim.cmd("colorscheme " .. theme_name)
end
