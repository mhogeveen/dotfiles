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
      cmd = { 'MasonUpdate' },
      lazy = true,
      config = {
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
      config = false,
    },
    {
      --- https://github.com/VonHeikemen/lsp-zero.nvim
      'VonHeikemen/lsp-zero.nvim',
      branch = 'v3.x',
      lazy = true,
      config = false,
      init = function()
        -- Disable automatic setup, we are doing it manually
        vim.g.lsp_zero_extend_cmp = 0
        vim.g.lsp_zero_extend_lspconfig = 0
        vim.g.lsp_zero_ui_float_border = 'single'
      end,
    },
  },
  cmd = { 'LspInfo', 'Mason' },
  event = { 'BufReadPre', 'BufNewFile' },
  config = function()
    local lsp_zero = require 'lsp-zero'
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
    lsp_zero.extend_lspconfig()

    lsp_zero.configure('lua_ls', {
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
    })

    --- rust-analyzer is installed via rustup
    lsp_zero.configure('rust_analyzer', {
      server = {
        standalone = true,
      },
    })

    lsp_zero.configure('stylelint_lsp', {
      filetypes = { 'css', 'less', 'scss', 'sass' },
    })

    lsp_zero.setup_servers(ensure_installed.ls)

    lsp_zero.on_attach(function(_, bufnr)
      ---@param keymap string
      ---@param func function
      local function set_buf_keymap(keymap, func)
        vim.keymap.set('n', keymap, func, { buffer = bufnr, noremap = true, silent = true })
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
    end)

    -- Set window styles
    require('lspconfig.ui.windows').default_options.border = 'single'

    -- Configure vim diagnostics
    vim.diagnostic.config {
      virtual_text = false,
      update_in_insert = true,
    }
  end,
}
