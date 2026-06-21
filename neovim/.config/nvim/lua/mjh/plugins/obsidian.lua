---@type LazySpec
return {
  'obsidian-nvim/obsidian.nvim',
  version = '*', -- use latest release, remove to use latest commit
  cmd = { 'Obsidian' },
  keys = {
    {
      '<leader>od',
      ':Obsidian today<CR>',
      mode = 'n',
      desc = '(O)bsidian (D)aily',
      noremap = true,
      silent = true,
    },
    {
      '<leader>oc',
      ':Obsidian toggle_checkbox<CR>',
      mode = 'n',
      desc = '(O)bsidian toggle_(C)heckbox',
      noremap = true,
      silent = true,
    },
    {
      '<leader>ol',
      ':Obsidian follow_link<CR>',
      mode = 'n',
      desc = '(O)bsidian follow_(L)ink',
      noremap = true,
      silent = true,
    },
    {
      '<leader>or',
      ':Obsidian search<CR>',
      mode = 'n',
      desc = '(O)bsidian (S)earch',
      noremap = true,
      silent = true,
    },
    {
      '<leader>of',
      ':Obsidian quick_switch<CR>',
      mode = 'n',
      desc = '(O)bsidian (F)ind',
      noremap = true,
      silent = true,
    },
  },
  opts = function()
    local Obsidian = require 'mjh.utils.obsidian'

    ---@module 'obsidian'
    ---@type obsidian.config
    return {
      legacy_commands = false, -- this will be removed in 4.0.0
      workspaces = {
        {
          name = 'Library of Wan Shi Tong',
          path = Obsidian.resolve_obsidian_vault(Obsidian.VaultType.Main).absolute_path,
        },
      },
      picker = {
        name = 'fzf-lua',
      },
      daily_notes = {
        folder = 'Work/Daily',
      },
      checkbox = {
        enabled = true,
        create_new = true,
        order = { ' ', 'x' },
      },
      open = {
        use_advanced_uri = true,
      },
      ui = {
        enable = true,
        enabled = true,
        ignore_conceal_warn = false,
        update_debounce = 200,
        max_file_length = 5000,
        checkboxes = {
          [' '] = { char = '󰄱', hl_group = 'obsidiantodo' },
          ['~'] = { char = '󰰱', hl_group = 'obsidiantilde' },
          ['!'] = { char = '', hl_group = 'obsidianimportant' },
          ['>'] = { char = '', hl_group = 'obsidianrightarrow' },
          ['x'] = { char = '󰱒', hl_group = 'obsidiandone' },
        },
        bullets = { char = '•', hl_group = 'ObsidianBullet' },
        external_link_icon = { char = '', hl_group = 'ObsidianExtLinkIcon' },
        reference_text = { hl_group = 'ObsidianRefText' },
        highlight_text = { hl_group = 'ObsidianHighlightText' },
        tags = { hl_group = 'ObsidianTag' },
        block_ids = { hl_group = 'ObsidianBlockID' },
        hl_groups = {
          ObsidianTodo = { bold = false, link = 'Special' },
          ObsidianDone = { bold = false, link = 'String' },
          ObsidianRightArrow = { bold = false, fg = '#f78c6c' },
          ObsidianTilde = { bold = false, fg = '#ff5370' },
          ObsidianImportant = { bold = false, fg = '#d73128' },
          ObsidianBullet = { bold = false, fg = '#89ddff' },
          ObsidianRefText = { link = 'Question' },
          ObsidianExtLinkIcon = { link = 'Question' },
          ObsidianTag = { italic = false, fg = '#89ddff' },
          ObsidianBlockID = { italic = false, fg = '#89ddff' },
          ObsidianHighlightText = { bg = '#75662e' },
        },
      },
    }
  end,
}
