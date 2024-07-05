return {
  --- https://github.com/hrsh7th/nvim-cmp
  'hrsh7th/nvim-cmp',
  event = { 'InsertEnter', 'CmdlineEnter' },
  dependencies = {
    --- https://github.com/VonHeikemen/lsp-zero.nvim
    'VonHeikemen/lsp-zero.nvim',
    --- https://github.com/hrsh7th/cmp-buffer
    'hrsh7th/cmp-buffer',
    --- https://github.com/hrsh7th/cmp-path
    'hrsh7th/cmp-path',
    --- https://github.com/hrsh7th/cmp-nvim-lsp
    'hrsh7th/cmp-nvim-lsp',
    --- https://github.com/hrsh7th/cmp-nvim-lua
    'hrsh7th/cmp-nvim-lua',
    --- https://github.com/tzachar/cmp-tabnine
    'tzachar/cmp-tabnine',
  },
  opts = function()
    local cmp = require 'cmp'
    local lsp_zero = require 'lsp-zero'

    return {
      snippet = {
        expand = function(args)
          vim.snippet.expand(args.body)
        end,
      },
      mapping = cmp.mapping.preset.insert {
        ['<C-Space>'] = cmp.mapping.complete(),
        ['<C-n>'] = cmp.mapping.select_next_item(),
        ['<C-p>'] = cmp.mapping.select_prev_item(),
        ['<C-y>'] = cmp.mapping.confirm { select = true },
        ['<C-e>'] = cmp.mapping.abort(),
      },
      sources = cmp.config.sources {
        { name = 'nvim_lua' },
        { name = 'nvim_lsp' },
        { name = 'cmp_tabnine' },
        { name = 'path' },
        { name = 'buffer', keyword_length = 5 },
      },
      formatting = lsp_zero.cmp_format(),
      window = {
        completion = cmp.config.window.bordered {
          border = 'single',
          winhighlight = 'Normal:Normal,FloatBorder:TelescopeBorder,CursorLine:Visual,Search:None',
        },
        documentation = cmp.config.window.bordered {
          border = 'single',
          winhighlight = 'Normal:Normal,FloatBorder:TelescopeBorder,CursorLine:Visual,Search:None',
        },
      },
    }
  end,
  config = function(_, opts)
    require('lsp-zero').extend_cmp()
    require('cmp').setup(opts)
  end,
}
