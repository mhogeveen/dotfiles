local status_mason, mason = pcall(require, "mason")
if not status_mason then
  return
end

local status_masonlsp, mason_lsp = pcall(require, "mason-lspconfig")
if not status_masonlsp then
  return
end

local status_lsp, lsp = pcall(require, "lspconfig")
if not status_lsp then
  return
end

-- Create default server config table
local default_server_config = {
  on_attach = require("mjh.lsp.on_attach"),
  flags = { debounce_text_changes = 150 },
  capabilities = require("mjh.lsp.capabilities"),
}

-- Setup mason and mason-lspconfig
-- Default handlers for each installed server
-- and override handlers for servers with custom config
mason.setup()
mason_lsp.setup()
mason_lsp.setup_handlers({
  -- Default server config
  function(server_name)
    lsp[server_name].setup(default_server_config)
  end,

  -- Lua config
  ["sumneko_lua"] = function()
    local lua_server_config = require("mjh.lsp.configs.lua")
    lsp["sumneko_lua"].setup(vim.tbl_deep_extend("force", lua_server_config, default_server_config))
  end,
})

-- Configure vim diagnostics
vim.diagnostic.config(require("mjh.lsp.configs.diagnostics"))
