return {
  'theblob42/drex.nvim',
  dependencies = {
    'kyazdani42/nvim-web-devicons',
  },
  cmd = { 'DrexDrawerOpen', 'DrexDrawerClose', 'DrexDrawerToggle' },
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
          ['l'] = { '<cmd>lua require("drex.elements").expand_element()<CR>', {
            desc = 'expand element',
          } },
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
          ['<C-x>'] = { '<cmd>lua require("drex.elements").open_file("sp")<CR>', {
            desc = 'open file in split',
          } },
          ['<C-r>'] = { '<cmd>lua require("drex").reload_directory()<CR>', {
            desc = 'reload',
          } },
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
          ['s'] = { '<cmd>lua require("drex.actions.stats").stats()<CR>', {
            desc = 'show element stats',
          } },
          ['a'] = { '<cmd>lua require("drex.actions.files").create()<CR>', {
            desc = 'create element',
          } },
          ['d'] = { '<cmd>lua require("drex.actions.files").delete("line")<CR>', {
            desc = 'delete element',
          } },
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
          ['r'] = { '<cmd>lua require("drex.actions.files").rename()<CR>', {
            desc = 'rename element',
          } },
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
          ['cc'] = { '<cmd>lua require("drex.clipboard").clear_clipboard()<CR>', {
            desc = 'clear clipboard',
          } },
          ['cs'] = {
            '<cmd>lua require("drex.clipboard").open_clipboard_window()<CR>',
            {
              desc = 'edit clipboard',
            },
          },
          ['y'] = { '<cmd>lua require("drex.actions.text").copy_name()<CR>', {
            desc = 'copy element name',
          } },
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
          ['d'] = { ':lua require("drex.actions.files").delete("visual")<CR>', {
            desc = 'delete elements',
          } },
          ['r'] = {
            ':lua require("drex.actions.files").multi_rename("visual")<CR>',
            {
              desc = 'rename elements',
            },
          },
          ['M'] = { ':DrexMark<CR>', { desc = 'mark elements' } },
          ['u'] = { ':DrexUnmark<CR>', { desc = 'unmark elements' } },
          ['m'] = { ':DrexToggle<CR>', { desc = 'toggle elements' } },
          ['y'] = { ':lua require("drex.actions.text").copy_name(true)<CR>', {
            desc = 'copy element names',
          } },
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
}
