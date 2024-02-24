return {
  --- https://github.com/hrsh7th/nvim-cmp
  'hrsh7th/nvim-cmp',
  event = 'InsertEnter',
  dependencies = {
    --- https://github.com/L3MON4D3/LuaSnip
    'L3MON4D3/LuaSnip',
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
    --- https://github.com/saadparwaiz1/cmp_luasnip
    'saadparwaiz1/cmp_luasnip',
    --- https://github.com/tzachar/cmp-tabnine
    'tzachar/cmp-tabnine',
    --- https://github.com/rafamadriz/friendly-snippets
    'rafamadriz/friendly-snippets',
  },
  opts = function()
    local cmp = require 'cmp'
    local luasnip = require 'luasnip'
    local lsp_zero = require 'lsp-zero'

    return {
      snippet = {
        expand = function(args)
          luasnip.lsp_expand(args.body)
        end,
      },
      mapping = cmp.mapping.preset.insert {
        ['<C-Space>'] = cmp.mapping.complete(),
        ['<C-n>'] = cmp.mapping.select_next_item(),
        ['<C-p>'] = cmp.mapping.select_prev_item(),
        ['<CR>'] = cmp.mapping.confirm {
          select = false,
          behavior = cmp.ConfirmBehavior.Insert,
        },
      },
      sources = cmp.config.sources {
        { name = 'nvim_lua' },
        { name = 'nvim_lsp' },
        { name = 'cmp_tabnine' },
        { name = 'luasnip' },
        { name = 'path' },
        { name = 'buffer',     keyword_length = 5 },
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
