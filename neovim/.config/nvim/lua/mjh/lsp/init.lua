local ok, lspconfig = pcall(require, "lspconfig")
if not ok then
  print("'nvim-lspconfig' is not installed")
  return
end

-- Set up UI styling
require("mjh.lsp.ui")

-- Set up LSP's
local merge_configs = require("mjh.lsp.configs.default")

lspconfig.bashls.setup(merge_configs({}))
lspconfig.cssls.setup(merge_configs({}))
lspconfig.eslint.setup(merge_configs({}))
lspconfig.html.setup(merge_configs({}))
lspconfig.intelephense.setup(merge_configs({}))
lspconfig.jsonls.setup(merge_configs({}))
lspconfig.lua_ls.setup(merge_configs(require("mjh.lsp.configs.lua_ls")))
lspconfig.marksman.setup(merge_configs({}))
lspconfig.tsserver.setup(merge_configs({}))
lspconfig.rust_analyzer.setup(merge_configs({ server = { standalone = true } }))
lspconfig.stylelint_lsp.setup(merge_configs({ filetypes = { "css", "less", "scss", "vue" } }))
