---@type LazySpec
return {
  --- https://github.com/neovim/nvim-lspconfig
  'neovim/nvim-lspconfig',
  lazy = true,
  dependencies = {
    {
      --- https://github.com/folke/neodev.nvim
      'folke/neodev.nvim',
      lazy = true,
      opts = {},
    },
    {
      --- https://github.com/williamboman/mason.nvim
      'williamboman/mason.nvim',
      dependencies = { 'Zeioth/mason-extra-cmds', opts = {} },
      lazy = true,
      opts = {
        ui = {
          border = 'single',
          height = 0.8,
          width = 0.8,
        },
      },
    },
    {
      --- https://github.com/williamboman/mason-lspconfig.nvim
      'williamboman/mason-lspconfig.nvim',
      lazy = true,
      opts = {},
    },
    {
      --- https://github.com/Saghen/blink.cmp
      'saghen/blink.cmp',
    },
  },
  cmd = { 'LspInfo', 'Mason' },
  event = { 'BufReadPre', 'BufNewFile' },
  config = function()
    local lspconfig = require 'lspconfig'
    local mason = require 'mason'
    local mason_lspconfig = require 'mason-lspconfig'

    local ensure_installed = {
      dap = {},
      formatters = {
        'prettierd',
        'stylua',
      },
      linters = {},
      ls = {
        'bashls',
        'biome',
        'cssls',
        'eslint',
        'gopls',
        'html',
        'intelephense',
        'jsonls',
        'lua_ls',
        'marksman',
        'stylelint_lsp',
        'ts_ls',
        'terraformls',
        'tailwindcss',
      },
    }

    mason.setup()
    mason_lspconfig.setup {
      ensure_installed = vim.tbl_extend(
        'force',
        ensure_installed.dap,
        ensure_installed.formatters,
        ensure_installed.linters,
        ensure_installed.ls
      ),
    }

    mason_lspconfig.setup_handlers {
      --- default handler
      function(server_name)
        lspconfig[server_name].setup {}
      end,

      ['lua_ls'] = function(server_name)
        lspconfig[server_name].setup {
          settings = {
            Lua = {
              runtime = {
                version = 'LuaJIT',
              },
              diagnostics = {
                globals = { 'vim', 'hs' },
              },
              workspace = {
                checkThirdParty = false,
                library = {
                  '${3rd}/luv/library',
                  unpack(vim.api.nvim_get_runtime_file('', true)),
                },
              },
              telemetry = false,
            },
          },
        }
      end,

      ['rust_analyzer'] = function()
        lspconfig['rust_analyzer'].setup {
          server = {
            standalone = true,
          },
        }
      end,

      ['stylelint_lsp'] = function()
        lspconfig['stylelint_lsp'].setup {
          filetypes = { 'css', 'less', 'scss', 'sass' },
        }
      end,
    }

    vim.api.nvim_create_autocmd('LspAttach', {
      desc = 'LSP actions',
      callback = function(event)
        ---@param keymap string
        ---@param func function
        local function set_buf_keymap(keymap, func)
          vim.keymap.set('n', keymap, func, { buffer = event.buf, noremap = true, silent = true })
        end

        set_buf_keymap('gD', vim.lsp.buf.declaration)
        set_buf_keymap('gh', vim.lsp.buf.hover)
        set_buf_keymap('gd', vim.lsp.buf.definition)
        set_buf_keymap('gt', vim.lsp.buf.type_definition)
        set_buf_keymap('gi', require('fzf-lua').lsp_implementations)
        set_buf_keymap('gs', vim.lsp.buf.signature_help)
        set_buf_keymap('gS', require('fzf-lua').lsp_document_symbols)
        set_buf_keymap('gr', require('fzf-lua').lsp_references)
        set_buf_keymap('gR', vim.lsp.buf.rename)
        set_buf_keymap('[d', vim.diagnostic.goto_prev)
        set_buf_keymap('gl', vim.diagnostic.open_float)
        set_buf_keymap(']d', vim.diagnostic.goto_next)
        set_buf_keymap('ga', require('fzf-lua').lsp_code_actions)
      end,
    })

    vim.lsp.handlers['textDocument/hover'] = vim.lsp.with(vim.lsp.handlers.hover, { border = 'single' })

    vim.lsp.handlers['textDocument/signatureHelp'] =
      vim.lsp.with(vim.lsp.handlers.signature_help, { border = 'single' })

    vim.diagnostic.config {
      float = { border = 'single' },
      virtual_text = false,
      update_in_insert = true,
    }

    lspconfig.util.default_config.capabilities = vim.tbl_deep_extend(
      'force',
      lspconfig.util.default_config.capabilities,
      require('blink.cmp').get_lsp_capabilities()
    )
  end,
}
