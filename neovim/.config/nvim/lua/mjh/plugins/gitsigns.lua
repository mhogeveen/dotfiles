---@type LazySpec
return {
  --- https://github.com/lewis6991/gitsigns.nvim
  'lewis6991/gitsigns.nvim',
  event = 'VeryLazy',
  keys = {
    {
      ']c',
      function()
        if vim.wo.diff then
          vim.cmd.normal { '[c', bang = true }
        else
          require('gitsigns').nav_hunk 'prev'
        end
      end,
      mode = { 'n' },
      desc = 'Move to next hunk',
    },
    {
      '[c',
      function()
        if vim.wo.diff then
          vim.cmd.normal { '[c', bang = true }
        else
          require('gitsigns').nav_hunk 'prev'
        end
      end,
      mode = { 'n' },
      desc = 'Move to prev hunk',
    },
  },
  opts = function()
    local git_icons = require('mjh.utils.icons').gitsigns

    return {
      signs = {
        add = {
          text = git_icons.add,
        },
        change = {
          text = git_icons.change,
        },
        delete = {
          text = git_icons.delete,
        },
        topdelete = {
          text = git_icons.topdelete,
        },
        changedelete = {
          text = git_icons.changedelete,
        },
        untracked = {
          text = git_icons.untracked,
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
      attach_to_untracked = false,
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
    }
  end,
}
