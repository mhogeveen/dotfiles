return {
  'mhartington/formatter.nvim',
  event = 'BufEnter',
  config = false,
  init = function()
    require('formatter').setup {
      logging = true,
      log_level = vim.log.levels.WARN,
      filetype = {
        html = {
          require('formatter.filetypes.html').prettierd,
        },
        css = {
          require('formatter.filetypes.css').prettierd,
        },
        json = {
          require('formatter.filetypes.json').prettierd,
        },
        jsonc = {
          require('formatter.filetypes.json').prettierd,
        },
        yaml = {
          require('formatter.filetypes.json').prettierd,
        },
        graphql = {
          require('formatter.filetypes.json').prettierd,
        },
        javascript = {
          require('formatter.filetypes.javascript').prettierd,
        },
        javascriptreact = {
          require('formatter.filetypes.javascriptreact').prettierd,
        },
        typescript = {
          require('formatter.filetypes.typescript').prettierd,
        },
        typescriptreact = {
          require('formatter.filetypes.typescriptreact').prettierd,
        },
        lua = {
          require('formatter.filetypes.lua').stylua,
        },
        rust = {
          require('formatter.filetypes.rust').rustfmt,
        },
        php = {
          function()
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
        },
        ['*'] = {
          function()
            local defaults = require 'formatter.defaults'
            local util = require 'formatter.util'

            -- Do not remove trailing whitespace in markdown files
            if util.get_current_buffer_file_extension() == 'md' then
              return
            end

            util.withl(defaults.sed, '[ \t]*$')
          end,
        },
      },
    }
  end,
}
