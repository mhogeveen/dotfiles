return {
  'neovim/nvim-lspconfig',
  dependencies = {
    'hrsh7th/cmp-nvim-lsp',
  },
  cmd = 'LspInfo',
  event = { 'BufReadPre', 'BufNewFile' },
  config = function()
    local lsp_zero = require 'lsp-zero'

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
        },
      },
    })

    lsp_zero.configure('rust_analyzer', {
      server = {
        standalone = true,
      },
    })

    lsp_zero.configure('stylelint_lsp', {
      filetypes = { 'css', 'less', 'scss', 'sass' },
    })

    lsp_zero.setup_servers {
      'bashls',
      'cssls',
      'eslint',
      'html',
      'intelephense',
      'jsonls',
      'lua_ls',
      'marksman',
      'tsserver',
      'rust_analyzer',
      'stylelint_lsp',
    }

    lsp_zero.on_attach(function(_, bufnr)
      local opts = { noremap = true, silent = true }
      local function set_buf_keymap(keymap, command)
        vim.api.nvim_buf_set_keymap(bufnr, 'n', keymap, command, opts)
      end

      set_buf_keymap('gD', ':lua vim.lsp.buf.declaration()<CR>')
      set_buf_keymap('gh', ':lua vim.lsp.buf.hover()<CR>')
      set_buf_keymap('gd', ':lua vim.lsp.buf.definition()<CR>')
      set_buf_keymap('gt', ':lua vim.lsp.buf.type_definition()<CR>')
      set_buf_keymap('gi', ':Telescope lsp_implementations<CR>')
      set_buf_keymap('gs', ':lua vim.lsp.buf.signature_help()<CR>')
      set_buf_keymap('gS', ':Telescope lsp_document_symbols<CR>')
      set_buf_keymap('gr', ':Telescope lsp_references<CR>')
      set_buf_keymap('gR', ':lua vim.lsp.buf.rename()<CR>')
      set_buf_keymap('[d', ':lua vim.diagnostic.goto_prev()<CR>')
      set_buf_keymap('gl', ':lua vim.diagnostic.open_float()<CR>')
      set_buf_keymap(']d', ':lua vim.diagnostic.goto_next()<CR>')
      set_buf_keymap('gf', ':lua vim.lsp.buf.format()<CR>')
      set_buf_keymap('ga', ':lua vim.lsp.buf.code_action()<CR>')
    end)

    lsp_zero.format_on_save {
      format_opts = {
        async = false,
        timeout_ms = 10000,
        formatting_options = {
          -- @see https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#formattingOptions
          insertSpaces = false,
          trimTrailingWhitespace = true,
          insertFinalNewline = false,
          trimFinalNewlines = true,
        },
      },
      servers = {
        ['stylelint_lsp'] = { 'css', 'less', 'scss', 'sass' },
      },
    }

    -- Set window styles
    require('lspconfig.ui.windows').default_options.border = 'single'

    -- Configure vim diagnostics
    vim.diagnostic.config {
      virtual_text = true,
      update_in_insert = true,
    }
  end,
}
