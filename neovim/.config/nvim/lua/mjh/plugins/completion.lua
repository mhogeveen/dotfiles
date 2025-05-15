---@type LazySpec
return {
  --- https://github.com/Saghen/blink.cmp
  'saghen/blink.cmp',
  version = 'v0.*',

  ---@module 'blink.cmp'
  ---@type blink.cmp.Config
  opts = {
    keymap = {
      preset = 'default',
      ['<C-.>'] = { 'show', 'show_documentation', 'hide_documentation' },
    },
    cmdline = {
      keymap = {
        preset = 'cmdline',
        ['<C-.>'] = { 'show', 'hide' },
      },
    },
    completion = {
      accept = {
        auto_brackets = {
          enabled = false,
        },
      },
      list = {
        selection = { preselect = true, auto_insert = false },
      },
      menu = {
        border = 'single',
        draw = {
          columns = { { 'label', 'label_description', gap = 1 }, { 'kind_icon', 'kind', gap = 1 } },
        },
      },
      documentation = {
        auto_show = true,
        window = {
          border = 'single',
        },
      },
      ghost_text = {
        enabled = true,
      },
    },
    signature = {
      enabled = false,
      window = {
        border = 'single',
      },
    },
    sources = {
      default = { 'lsp', 'path', 'buffer' },
      providers = {
        cmdline = {
          min_keyword_length = 2,
        },
        lsp = {
          enabled = function()
            return not vim.tbl_contains({ 'http', 'rest' }, vim.bo.filetype)
          end,
        },
      },
    },
    appearance = {
      use_nvim_cmp_as_default = false,
      nerd_font_variant = 'mono',
    },
  },
}
