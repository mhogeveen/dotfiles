local orgfiles_base_dir = '~/git/orgfiles'
local orgfiles_locs = {
  base = orgfiles_base_dir,
  personal = orgfiles_base_dir .. '/personal.org',
  work = orgfiles_base_dir .. '/work.org',
  inbox = orgfiles_base_dir .. '/inbox.org',
}

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
          org_agenda_files = orgfiles_locs.base .. '/**/*',
          org_default_notes_file = orgfiles_locs.inbox,
          org_todo_keywords = { 'TODO(t)', 'INPROGRESS(i)', 'WAITING(w)', '|', 'DONE(d)' },
          org_todo_repeat_to_state = 'TODO',
          win_split_mode = { 'float', 0.8 },
          org_startup_folded = 'showeverything',
          org_todo_keyword_faces = {
            INPROGRESS = ':foreground #F9E2AF',
            WAITING = ':foreground #FAB387',
          },
          org_hide_emphasis_markers = true,
          org_ellipsis = '[ ... ]',
          org_log_into_drawer = 'LOGBOOK',
          org_startup_indented = true,
          org_adapt_indentation = true,
          org_indent_mode_turns_off_org_adapt_indentation = false,
          org_indent_mode_turns_on_hiding_stars = false,
          org_id_link_to_org_use_id = true,
          org_agenda_custom_commands = {
            p = {
              description = 'Personal Agenda',
              types = {
                {
                  type = 'agenda',
                  org_agenda_overriding_header = 'Personal Agenda',
                  org_agenda_files = { orgfiles_locs.personal },
                },
              },
            },
            w = {
              description = 'Work Agenda',
              types = {
                {
                  type = 'agenda',
                  org_agenda_overriding_header = 'Work Agenda',
                  org_agenda_files = { orgfiles_locs.work },
                },
              },
            },
          },
          org_capture_templates = {
            t = {
              description = 'Task',
              subtemplates = {
                i = {
                  description = 'Inbox',
                  template = '** TODO %?\n %u',
                },
                p = {
                  description = 'Personal',
                  template = '** TODO %?\n %u',
                  target = orgfiles_locs.personal,
                },
                w = {
                  description = 'Work',
                  template = '** TODO %?\n %u',
                  target = orgfiles_locs.work,
                },
              },
            },
          },

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
    directory = orgfiles_locs.base,
  },
}
