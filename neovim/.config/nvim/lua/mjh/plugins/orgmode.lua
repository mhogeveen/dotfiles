---@type LazySpec
return {
  --- https://github.com/chipsenkbeil/org-roam.nvim
  'chipsenkbeil/org-roam.nvim',
  tag = '0.2.0',
  dependencies = {
    {
      --- https://github.com/nvim-orgmode/orgmode
      'nvim-orgmode/orgmode',
      tag = '0.7.0',
      dependencies = {
        --- https://github.com/danilshvalov/org-modern.nvim
        'danilshvalov/org-modern.nvim',
      },
      ft = { 'org' },
      config = function()
        local Menu = require 'org-modern.menu'
        -- Setup orgmode
        require('orgmode').setup {
          org_agenda_files = '~/git/orgfiles/**/*',
          org_default_notes_file = '~/git/orgfiles/refile.org',
          win_split_mode = 'float',
          org_startup_folded = 'showeverything',
          org_todo_keywords = { 'TODO', 'INPROGRESS', 'WAITING', '|', 'DONE' },
          org_todo_keyword_faces = {
            INPROGRESS = ':foreground #F9E2AF',
            WAITING = ':foreground #FAB387',
          },
          org_log_into_drawer = 'LOGBOOK',
          org_startup_indented = true,
          org_adapt_indentation = true,
          org_indent_mode_turns_off_org_adapt_indentation = false,
          org_indent_mode_turns_on_hiding_stars = false,
          org_hide_emphasis_markers = true,

          ui = {
            menu = {
              handler = function(data)
                data.title = ' ' .. data.title .. ' '
                Menu:new():open(data)
              end,
            },
          },
        }

        -- Experimental LSP support
        vim.lsp.enable 'org'
      end,
    },
  },
  opts = {
    directory = '~/git/orgfiles',
  },
}
