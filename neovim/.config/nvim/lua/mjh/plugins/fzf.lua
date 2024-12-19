---@param projects_dir string
local function fzf_projects(projects_dir)
  local opts = {
    prompt = 'Projects > ',
    actions = {
      ['default'] = function(selected)
        vim.cmd('cd ' .. selected[1])
      end,
      ['ctrl-d'] = {
        fn = function()
          vim.cmd 'cd ~/.dotfiles'
        end,
        header = 'open dotfiles',
      },
    },
    fn_transform = function(option)
      return projects_dir .. '/' .. option
    end,
  }

  opts = require('fzf-lua').config.normalize_opts(opts, {}) -- Inherit global opts (highlights, etc)
  opts = require('fzf-lua').core.set_header(opts, { 'actions' }) -- Setup header string from actions tbl

  require('fzf-lua').fzf_exec('ls ' .. projects_dir, opts)
end

---@type LazySpec
return {
  --- https://github.com/ibhagwan/fzf-lua
  'ibhagwan/fzf-lua',
  dependencies = { 'nvim-tree/nvim-web-devicons' },
  keys = {
    {
      'ff',
      function()
        require('fzf-lua').files()
      end,
      mode = 'n',
      desc = 'fzf - find files',
      noremap = true,
      silent = true,
    },
    {
      'fg',
      function()
        require('fzf-lua').git_files()
      end,
      mode = 'n',
      desc = 'fzf - git files',
      noremap = true,
      silent = true,
    },
    {
      'fr',
      function()
        require('fzf-lua').live_grep()
      end,
      mode = 'n',
      desc = 'fzf - grep',
      noremap = true,
      silent = true,
    },
    {
      'fc',
      function()
        require('fzf-lua').grep_cword()
      end,
      mode = 'n',
      desc = 'fzf - grep cword',
      noremap = true,
      silent = true,
    },
    {
      'fb',
      function()
        require('fzf-lua').buffers()
      end,
      mode = 'n',
      desc = 'fzf - buffers',
      noremap = true,
      silent = true,
    },
    {
      'fp',
      function()
        fzf_projects '~/repos'
      end,
      mode = 'n',
      desc = 'fzf - projects',
      noremap = true,
      silent = true,
    },
  },
  cmd = 'FzfLua',
  opts = function()
    local actions = require 'fzf-lua.actions'
    return {
      winopts = {
        border = 'single',
      },
      files = {
        formatter = 'path.filename_first',
      },
      buffers = {
        formatter = 'path.filename_first',
      },
      grep = {
        formatter = 'path.filename_first',
        actions = {
          ['ctrl-q'] = {
            fn = actions.file_edit_or_qf,
            prefix = 'select-all+',
          },
        },
        rg_opts = '--sort-files --hidden --column --line-number --no-heading '
          .. "--color=always --smart-case -g '!{.git,node_modules}/*'",
      },
      keymap = {
        builtin = {
          false,
          ['<C-h>'] = 'toggle-help',
        },
        fzf = {
          false,
          ['ctrl-u'] = 'half-page-up',
          ['ctrl-d'] = 'half-page-down',
        },
      },
    }
  end,
  config = function(_, opts)
    local fzf_lua = require 'fzf-lua'
    fzf_lua.setup(opts)
    fzf_lua.register_ui_select()
  end,
}
