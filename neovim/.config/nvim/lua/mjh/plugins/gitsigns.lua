return {
  --- https://github.com/lewis6991/gitsigns.nvim
  'lewis6991/gitsigns.nvim',
  event = 'VeryLazy',
  opts = function()
    local git_icons = require('mjh.utils.icons').gitsigns

    return {
      signs = {
        add = { hl = 'GitSignsAdd', text = git_icons.add, numhl = 'GitSignsAddNr', linehl = 'GitSignsAddLn' },
        change = {
          hl = 'GitSignsChange',
          text = git_icons.change,
          numhl = 'GitSignsChangeNr',
          linehl = 'GitSignsChangeLn',
        },
        delete = {
          hl = 'GitSignsDelete',
          text = git_icons.delete,
          numhl = 'GitSignsDeleteNr',
          linehl = 'GitSignsDeleteLn',
        },
        topdelete = {
          hl = 'GitSignsDelete',
          text = git_icons.topdelete,
          numhl = 'GitSignsDeleteNr',
          linehl = 'GitSignsDeleteLn',
        },
        changedelete = {
          hl = 'GitSignsChange',
          text = git_icons.changedelete,
          numhl = 'GitSignsChangeNr',
          linehl = 'GitSignsChangeLn',
        },
        untracked = {
          hl = 'GitSignsUntracked',
          text = git_icons.untracked,
          numhl = 'GitSignsUntrackedNr',
          linehl = 'GitSignsUntrackedLn',
        },
      },
      signcolumn = true, -- Toggle with `:Gitsigns toggle_signs`
      numhl = false, -- Toggle with `:Gitsigns toggle_numhl`
      linehl = false, -- Toggle with `:Gitsigns toggle_linehl`
      word_diff = false, -- Toggle with `:Gitsigns toggle_word_diff`
      watch_gitdir = {
        interval = 1000,
        follow_files = true,
      },
      attach_to_untracked = true,
      current_line_blame = false, -- Toggle with `:Gitsigns toggle_current_line_blame`
      current_line_blame_opts = {
        virt_text = true,
        virt_text_pos = 'eol', -- 'eol' | 'overlay' | 'right_align'
        delay = 1000,
        ignore_whitespace = false,
      },
      current_line_blame_formatter = '<author>, <author_time:%Y-%m-%d> - <summary>',
      sign_priority = 6,
      update_debounce = 100,
      status_formatter = nil, -- Use default
      max_file_length = 40000, -- Disable if file is longer than this (in lines)
      preview_config = {
        -- Options passed to nvim_open_win
        border = 'single',
        style = 'minimal',
        relative = 'cursor',
        row = 0,
        col = 1,
      },
      yadm = {
        enable = false,
      },
    }
  end,
}
