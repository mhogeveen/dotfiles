return {
  'nvim-lualine/lualine.nvim',
  config = function()
    require('lualine').setup {
      options = {
        icons_enabled = true,
        theme = 'catppuccin', -- auto, catppuccin
        component_separators = { left = '', right = '' },
        -- section_separators = { left = "", right = "" },
        section_separators = { left = '', right = '' },
        disabled_filetypes = {
          statusline = {},
          winbar = {},
        },
        ignore_focus = {},
        always_divide_middle = true,
        globalstatus = true,
        refresh = {
          statusline = 1000,
          tabline = 1000,
          winbar = 1000,
        },
      },
      sections = {
        lualine_a = { 'mode' },
        lualine_b = {
          {
            'filename',
            file_status = false,
          },
        },
        lualine_c = {
          {
            'branch',
            -- icon = " ",
            icon = '',
          },
          {
            'diff',
            symbols = {
              -- added = " ",
              -- modified = " ",
              -- removed = " ",
              added = '+',
              modified = '~',
              removed = '-',
            },
          },
        },
        lualine_x = {},
        lualine_y = {
          'bo:filetype',
          -- {
          --   "filetype",
          --   colored = false,
          -- },
        },
        lualine_z = { 'location' },
      },
      inactive_sections = {
        lualine_a = {},
        lualine_b = {},
        lualine_c = { 'filename' },
        lualine_x = { 'location' },
        lualine_y = {},
        lualine_z = {},
      },
      tabline = {},
      winbar = {},
      inactive_winbar = {},
      extensions = {},
    }
  end,
}
