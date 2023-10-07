return {
  {
    'tzachar/cmp-tabnine',
    build = './install.sh',
  },
  {
    'hrsh7th/cmp-nvim-lsp',
  },
  {
    'hrsh7th/cmp-buffer',
  },
  {
    'hrsh7th/cmp-path',
  },
  {
    'hrsh7th/cmp-nvim-lua',
  },
  {
    'saadparwaiz1/cmp_luasnip',
    dependencies = {
      {
        'L3MON4D3/LuaSnip',
        dependencies = {
          'rafamadriz/friendly-snippets',
        },
      },
    },
  },
  {
    'hrsh7th/nvim-cmp',
    event = 'InsertEnter',
    dependencies = {
      'L3MON4D3/LuaSnip',
      'VonHeikemen/lsp-zero.nvim',
      'hrsh7th/cmp-buffer',
      'hrsh7th/cmp-path',
      'hrsh7th/cmp-nvim-lsp',
      'hrsh7th/cmp-nvim-lua',
      'saadparwaiz1/cmp_luasnip',
      'tzachar/cmp-tabnine',
    },
    config = function()
      local cmp = require 'cmp'
      local luasnip = require 'luasnip'
      local lsp_zero = require 'lsp-zero'

      lsp_zero.extend_cmp()

      cmp.setup {
        snippet = {
          expand = function(args)
            luasnip.lsp_expand(args.body)
          end,
        },
        mapping = cmp.mapping.preset.insert {
          ['<C-n>'] = cmp.mapping.select_next_item(),
          ['<C-p>'] = cmp.mapping.select_prev_item(),
          ['<CR>'] = cmp.mapping.confirm {
            select = false,
            behavior = cmp.ConfirmBehavior.Insert,
          },
        },
        sources = cmp.config.sources {
          { name = 'nvim_lsp' },
          { name = 'nvim_lua' },
          { name = 'cmp_tabnine' },
          { name = 'luasnip' },
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
  },
}
