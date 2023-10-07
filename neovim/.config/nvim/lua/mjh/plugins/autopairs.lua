return {
  'windwp/nvim-autopairs',
  dependencies = { 'hrsh7th/nvim-cmp' },
  event = 'InsertEnter',
  config = function()
    local autopairs = require 'nvim-autopairs'
    local rule = require 'nvim-autopairs.rule'
    local cmp_autopairs = require 'nvim-autopairs.completion.cmp'
    local cmp = require 'cmp'

    autopairs.setup {
      check_ts = true,
      ts_config = {
        lua = { 'string', 'source' },
        javascript = { 'string', 'template_string' },
        java = false,
      },
      disable_filetype = { 'TelescopePrompt', 'spectre_panel' },
      fast_wrap = {
        map = '<M-e>',
        chars = { '{', '[', '(', '"', "'", '`' },
        pattern = string.gsub([[ [%'%"%)%>%]%)%}%,] ]], '%s+', ''),
        offset = 0, -- Offset from pattern match
        end_key = '$',
        keys = 'qwertyuiopzxcvbnmasdfghjkl',
        highlight_grey = 'LineNr',
      },
    }

    autopairs.add_rules {
      rule('<', '>', { 'rust' }),
      rule('|', '|', { 'rust' }),
    }

    cmp.event:on('confirm_done', cmp_autopairs.on_confirm_done { map_char = { tex = '' } })
  end,
}
