local ok, lspconfig = pcall(require, "lspconfig")
if not ok then
  print("'nvim-lspconfig' is not installed")
  return
end

-- Set up UI styling
require('mjh.lsp.ui')

-- Set up LSP's
local merge_configs = require('mjh.lsp.configs.default')

lspconfig.bashls.setup({})
lspconfig.cssls.setup({})
lspconfig.eslint.setup({})
lspconfig.html.setup({})
lspconfig.intelephense.setup({})
lspconfig.jsonls.setup({})
lspconfig.lua_ls.setup(merge_configs(require('mjh.lsp.configs.lua_ls')))
lspconfig.marksman.setup({})
lspconfig.tsserver.setup({})
