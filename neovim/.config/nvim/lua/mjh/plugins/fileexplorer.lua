return {
  {
    --- https://github.com/stevearc/oil.nvim
    'stevearc/oil.nvim',
    enabled = false,
    keys = {
      {
        '<leader>ee',
        ':Oil<CR>',
        mode = 'n',
        desc = 'Open Oil',
        noremap = true,
        silent = true,
      },
      {
        '<leader>ef',
        ":lua require('oil').toggle_float()<CR>",
        mode = 'n',
        desc = 'Toggle Oil float',
        noremap = true,
        silent = true,
      },
    },
    opts = {
      -- Oil will take over directory buffers (e.g. `vim .` or `:e src/`)
      -- Set to false if you still want to use netrw.
      default_file_explorer = true,
      -- Id is automatically added at the beginning, and name at the end
      -- See :help oil-columns
      columns = {
        'icon',
        -- "permissions",
        -- "size",
        -- "mtime",
      },
      -- Buffer-local options to use for oil buffers
      buf_options = {
        buflisted = false,
        bufhidden = 'hide',
      },
      -- Window-local options to use for oil buffers
      win_options = {
        wrap = false,
        signcolumn = 'no',
        cursorcolumn = false,
        foldcolumn = '0',
        spell = false,
        list = false,
        conceallevel = 3,
        concealcursor = 'nvic',
      },
      -- Send deleted files to the trash instead of permanently deleting them (:help oil-trash)
      delete_to_trash = true,
      -- Skip the confirmation popup for simple operations
      skip_confirm_for_simple_edits = true,
      -- Selecting a new/moved/renamed file or directory will prompt you to save changes first
      prompt_save_on_select_new_entry = true,
      -- Oil will automatically delete hidden buffers after this delay
      -- You can set the delay to false to disable cleanup entirely
      -- Note that the cleanup process only starts when none of the oil buffers are currently displayed
      cleanup_delay_ms = 2000,
      -- Set to true to autosave buffers that are updated with LSP willRenameFiles
      -- Set to "unmodified" to only save unmodified buffers
      lsp_rename_autosave = 'unmodified',
      -- Constrain the cursor to the editable parts of the oil buffer
      -- Set to `false` to disable, or "name" to keep it on the file names
      constrain_cursor = 'editable',
      -- Keymaps in oil buffer. Can be any value that `vim.keymap.set` accepts OR a table of keymap
      -- options with a `callback` (e.g. { callback = function() ... end, desc = "", mode = "n" })
      -- Additionally, if it is a string that matches "actions.<name>",
      -- it will use the mapping at require("oil.actions").<name>
      -- Set to `false` to remove a keymap
      -- See :help oil-actions for a list of all available actions
      keymaps = {
        ['g?'] = 'actions.show_help',
        ['<CR>'] = 'actions.select',
        ['<C-v>'] = 'actions.select_vsplit',
        ['<C-s>'] = 'actions.select_split',
        -- ['<C-t>'] = 'actions.select_tab',
        ['p'] = 'actions.preview',
        ['q'] = 'actions.close',
        ['<Esc>'] = 'actions.close',
        ['r'] = 'actions.refresh',
        ['-'] = 'actions.parent',
        ['.'] = 'actions.open_cwd',
        ['`'] = 'actions.cd',
        ['~'] = 'actions.tcd',
        ['s'] = 'actions.change_sort',
        ['x'] = 'actions.open_external',
        -- ['g.'] = 'actions.toggle_hidden',
        -- ['g\\'] = 'actions.toggle_trash',
      },
      -- Set to false to disable all of the above keymaps
      use_default_keymaps = true,
      view_options = {
        -- Show files and directories that start with "."
        show_hidden = true,
        -- This function defines what is considered a "hidden" file
        is_hidden_file = function(name, bufnr)
          return vim.startswith(name, '.')
        end,
        -- This function defines what will never be shown, even when `show_hidden` is set
        is_always_hidden = function(name, bufnr)
          return false
        end,
        sort = {
          -- sort order can be "asc" or "desc"
          -- see :help oil-columns to see which columns are sortable
          { 'type', 'asc' },
          { 'name', 'asc' },
        },
      },
      -- Configuration for the floating window in oil.open_float
      float = {
        -- Padding around the floating window
        padding = 2,
        max_width = 80,
        max_height = 20,
        border = 'single',
        win_options = {
          winblend = 0,
        },
        -- This is the config that will be passed to nvim_open_win.
        -- Change values here to customize the layout
        override = function(conf)
          return conf
        end,
      },
      -- Configuration for the actions floating preview window
      preview = {
        -- Width dimensions can be integers or a float between 0 and 1 (e.g. 0.4 for 40%)
        -- min_width and max_width can be a single value or a list of mixed integer/float types.
        -- max_width = {100, 0.8} means "the lesser of 100 columns or 80% of total"
        max_width = 0.9,
        -- min_width = {40, 0.4} means "the greater of 40 columns or 40% of total"
        min_width = { 40, 0.4 },
        -- optionally define an integer/float for the exact width of the preview window
        width = nil,
        -- Height dimensions can be integers or a float between 0 and 1 (e.g. 0.4 for 40%)
        -- min_height and max_height can be a single value or a list of mixed integer/float types.
        -- max_height = {80, 0.9} means "the lesser of 80 columns or 90% of total"
        max_height = 0.9,
        -- min_height = {5, 0.1} means "the greater of 5 columns or 10% of total"
        min_height = { 5, 0.1 },
        -- optionally define an integer/float for the exact height of the preview window
        height = nil,
        border = 'single',
        win_options = {
          winblend = 0,
        },
        -- Whether the preview window is automatically updated when the cursor is moved
        update_on_cursor_moved = true,
      },
      -- Configuration for the floating progress window
      progress = {
        max_width = 0.9,
        min_width = { 40, 0.4 },
        width = nil,
        max_height = { 10, 0.9 },
        min_height = { 5, 0.1 },
        height = nil,
        border = 'single',
        minimized_border = 'none',
        win_options = {
          winblend = 0,
        },
      },
    },
  },
  {
    'theblob42/drex.nvim',
    enabled = true,
    event = 'VeryLazy',
    config = function()
      require('drex.config').configure {
        icons = {
          file_default = '',
          dir_open = '',
          dir_closed = '',
          link = '',
          others = '',
        },
        colored_icons = true,
        hide_cursor = false,
        hijack_netrw = true,
        sorting = function(a, b)
          local aname, atype = a[1], a[2]
          local bname, btype = b[1], b[2]

          local aisdir = atype == 'directory'
          local bisdir = btype == 'directory'

          if aisdir ~= bisdir then
            return aisdir
          end

          return aname < bname
        end,
        drawer = {
          side = 'right',
          default_width = 40,
          window_picker = {
            enabled = true,
            labels = 'abcdefghijklmnopqrstuvwxyz',
          },
        },
        disable_default_keybindings = true,
        keybindings = {
          ['n'] = {
            ['v'] = 'V',
            ['q'] = { '<cmd>DrexDrawerClose<CR>', { desc = 'close drawer' } },
            ['l'] = {
              '<cmd>lua require("drex.elements").expand_element()<CR>',
              {
                desc = 'expand element',
              },
            },
            ['h'] = {
              '<cmd>lua require("drex.elements").collapse_directory()<CR>',
              {
                desc = 'collapse directory',
              },
            },
            ['<C-v>'] = {
              '<cmd>lua require("drex.elements").open_file("vs")<CR>',
              {
                desc = 'open file in vsplit',
              },
            },
            ['<C-x>'] = {
              '<cmd>lua require("drex.elements").open_file("sp")<CR>',
              {
                desc = 'open file in split',
              },
            },
            ['<C-r>'] = {
              '<cmd>lua require("drex").reload_directory()<CR>',
              {
                desc = 'reload',
              },
            },
            ['gj'] = {
              '<cmd>lua require("drex.actions.jump").jump_to_next_sibling()<CR>',
              {
                desc = 'jump to next sibling',
              },
            },
            ['gk'] = {
              '<cmd>lua require("drex.actions.jump").jump_to_prev_sibling()<CR>',
              {
                desc = 'jump to prev sibling',
              },
            },
            ['gh'] = {
              '<cmd>lua require("drex.actions.jump").jump_to_parent()<CR>',
              {
                desc = 'jump to parent element',
              },
            },
            ['s'] = {
              '<cmd>lua require("drex.actions.stats").stats()<CR>',
              {
                desc = 'show element stats',
              },
            },
            ['a'] = {
              '<cmd>lua require("drex.actions.files").create()<CR>',
              {
                desc = 'create element',
              },
            },
            ['d'] = {
              '<cmd>lua require("drex.actions.files").delete("line")<CR>',
              {
                desc = 'delete element',
              },
            },
            ['D'] = {
              '<cmd>lua require("drex.actions.files").delete("clipboard")<CR>',
              {
                desc = 'delete (clipboard)',
              },
            },
            ['p'] = {
              '<cmd>lua require("drex.actions.files").copy_and_paste()<CR>',
              {
                desc = 'copy & paste (clipboard)',
              },
            },
            ['P'] = {
              '<cmd>lua require("drex.actions.files").cut_and_move()<CR>',
              {
                desc = 'cut & move (clipboard)',
              },
            },
            ['r'] = {
              '<cmd>lua require("drex.actions.files").rename()<CR>',
              {
                desc = 'rename element',
              },
            },
            ['R'] = {
              '<cmd>lua require("drex.actions.files").multi_rename("clipboard")<CR>',
              {
                desc = 'rename (clipboard)',
              },
            },
            ['/'] = {
              '<cmd>keepalt lua require("drex.actions.search").search()<CR>',
              {
                desc = 'search',
              },
            },
            ['M'] = { '<cmd>DrexMark<CR>', { desc = 'mark element' } },
            ['u'] = { '<cmd>DrexUnmark<CR>', { desc = 'unmark element' } },
            ['m'] = { '<cmd>DrexToggle<CR>', { desc = 'toggle element' } },
            ['cc'] = {
              '<cmd>lua require("drex.clipboard").clear_clipboard()<CR>',
              {
                desc = 'clear clipboard',
              },
            },
            ['cs'] = {
              '<cmd>lua require("drex.clipboard").open_clipboard_window()<CR>',
              {
                desc = 'edit clipboard',
              },
            },
            ['y'] = {
              '<cmd>lua require("drex.actions.text").copy_name()<CR>',
              {
                desc = 'copy element name',
              },
            },
            ['Y'] = {
              '<cmd>lua require("drex.actions.text").copy_relative_path()<CR>',
              {
                desc = 'copy element relative path',
              },
            },
            ['<C-y>'] = {
              '<cmd>lua require("drex.actions.text").copy_absolute_path()<CR>',
              {
                desc = 'copy element absolute path',
              },
            },
          },
          ['v'] = {
            ['d'] = {
              ':lua require("drex.actions.files").delete("visual")<CR>',
              {
                desc = 'delete elements',
              },
            },
            ['r'] = {
              ':lua require("drex.actions.files").multi_rename("visual")<CR>',
              {
                desc = 'rename elements',
              },
            },
            ['M'] = { ':DrexMark<CR>', { desc = 'mark elements' } },
            ['u'] = { ':DrexUnmark<CR>', { desc = 'unmark elements' } },
            ['m'] = { ':DrexToggle<CR>', { desc = 'toggle elements' } },
            ['y'] = {
              ':lua require("drex.actions.text").copy_name(true)<CR>',
              {
                desc = 'copy element names',
              },
            },
            ['Y'] = {
              ':lua require("drex.actions.text").copy_relative_path(true)<CR>',
              { desc = 'copy element relative paths' },
            },
            ['<C-y>'] = {
              ':lua require("drex.actions.text").copy_absolute_path(true)<CR>',
              {
                desc = 'copy element absolute paths',
              },
            },
          },
        },
        on_enter = function()
          vim.opt_local.number = false
          vim.opt_local.buflisted = false
          vim.opt_local.modifiable = false
          vim.opt_local.buftype = 'nofile'
          vim.opt_local.bufhidden = 'wipe'
        end,
        on_leave = nil,
      }

      vim.keymap.set('n', '<leader>e', function()
        if vim.tbl_contains({ 'drex', 'alpha', '', 'TelescopePrompt' }, vim.bo.filetype) then
          require('drex.drawer').toggle()
        else
          require('drex.drawer').find_element('%', true, true)
        end
      end, {
        noremap = true,
        silent = true,
      })

      vim.api.nvim_create_autocmd('BufEnter', {
        group = vim.api.nvim_create_augroup('DrexDrawerWindow', {}),
        pattern = '*',
        callback = function(args)
          if vim.api.nvim_get_current_win() == require('drex.drawer').get_drawer_window() then
            local is_drex_buffer = function(b)
              local ok, syntax = pcall(vim.api.nvim_buf_get_option, b, 'syntax')
              return ok and syntax == 'drex'
            end
            local prev_buf = vim.fn.bufnr '#'

            if is_drex_buffer(prev_buf) and not is_drex_buffer(args.buf) then
              vim.api.nvim_set_current_buf(prev_buf)
              vim.schedule(function()
                vim.cmd [['"]] -- restore former cursor position
              end)
            end
          end
        end,
      })
    end,
  },
}
