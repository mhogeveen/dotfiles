return {
  --- https://github.com/neovim/nvim-lspconfig
  'neovim/nvim-lspconfig',
  lazy = true,
  dependencies = {
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
        'tsserver',
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
      local opts = { noremap = true, silent = true }
      local function set_buf_keymap(keymap, command)
        vim.api.nvim_buf_set_keymap(bufnr, 'n', keymap, command, opts)
      end

      set_buf_keymap('gD', ':lua vim.lsp.buf.declaration()<CR>')
      set_buf_keymap('gh', ':lua vim.lsp.buf.hover()<CR>')
      set_buf_keymap('gd', ':lua vim.lsp.buf.definition()<CR>')
      set_buf_keymap('gt', ':lua vim.lsp.buf.type_definition()<CR>')
      set_buf_keymap('gi', ':FzfLua lsp_implementations<CR>')
      set_buf_keymap('gs', ':lua vim.lsp.buf.signature_help()<CR>')
      set_buf_keymap('gS', ':FzfLua lsp_document_symbols<CR>')
      set_buf_keymap('gr', ':FzfLua lsp_references<CR>')
      set_buf_keymap('gR', ':lua vim.lsp.buf.rename()<CR>')
      set_buf_keymap('[d', ':lua vim.diagnostic.goto_prev()<CR>')
      set_buf_keymap('gl', ':lua vim.diagnostic.open_float()<CR>')
      set_buf_keymap(']d', ':lua vim.diagnostic.goto_next()<CR>')
      -- set_buf_keymap('gf', ':lua vim.lsp.buf.format()<CR>')
      -- set_buf_keymap('ga', ':lua vim.lsp.buf.code_action()<CR>')
      set_buf_keymap('ga', ':FzfLua lsp_code_actions<CR>')
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
