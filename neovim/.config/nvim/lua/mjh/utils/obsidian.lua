local M = {}

---@enum mjh.utils.VaultType
M.VaultType = {
  Main = 'Main',
}

---@class mjh.utils.VaultConfig
---@field display_name string
---@field directory_name string

---@type table<mjh.utils.VaultType, mjh.utils.VaultConfig>
M.VaultConfigs = {
  [M.VaultType.Main] = {
    display_name = 'Library of Wan Shi Tong',
    directory_name = 'library-of-wan-shi-tong',
  },
}

---@class mjh.utils.ResolvedVault
---@field absolute_path string
---@field display_name string
---@field vault_type mjh.utils.VaultType

---@param name mjh.utils.VaultType
---@return mjh.utils.ResolvedVault?
M.resolve_obsidian_vault = function(name)
  local config = M.VaultConfigs[name]

  if not config then
    vim.notify('No config found for vault: ' .. name, vim.log.levels.ERROR)
    return nil
  end

  local absolute_path = vim.fn.expand '~' .. '/git/' .. config.directory_name

  if vim.fn.isdirectory(absolute_path) == 0 then
    vim.notify('Vault not found: ' .. absolute_path, vim.log.levels.ERROR)
    return nil
  end

  return {
    absolute_path = absolute_path,
    display_name = config.display_name,
    vault_type = name,
  }
end

return M
