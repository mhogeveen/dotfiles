return {
  'mhartington/formatter.nvim',
  event = 'BufEnter',
  config = false,
  init = function()
    local defaults = require 'formatter.defaults'
    local util = require 'formatter.util'

    require('formatter').setup {
      logging = true,
      log_level = vim.log.levels.WARN,
      filetype = {
        html = defaults.prettierd,
        css = defaults.prettierd,
        json = defaults.prettierd,
        jsonc = defaults.prettierd,
        yaml = defaults.prettierd,
        graphql = defaults.prettierd,
        javascript = defaults.prettierd,
        javascriptreact = defaults.prettierd,
        typescript = defaults.prettierd,
        typescriptreact = defaults.prettierd,
        lua = require('formatter.filetypes.lua').stylua,
        rust = require('formatter.filetypes.rust').rustfmt,
        php = function()
          return {
            exe = 'php-cs-fixer',
            args = {
              'fix',
              '--using-cache=no',
            },
            stdin = false,
            ignore_exitcode = true,
          }
        end,
        ['*'] = function()
          -- Do not remove trailing whitespace in markdown files
          if util.get_current_buffer_file_extension() == 'md' then
            return
          end

          util.withl(defaults.sed, '[ \t]*$')
        end,
      },
    }
  end,
}
