---@type LazySpec
return {
  -- https://github.com/catppuccin/nvim
  'catppuccin/nvim',
  name = 'catppuccin',
  priority = 1000,
  init = function()
    -- Automatically recompile after config change
    vim.api.nvim_create_autocmd('BufWritePost', {
      pattern = '**/plugins/colorscheme.lua',
      callback = function()
        vim.cmd 'CatppuccinCompile'
      end,
    })
    -- Set the colorscheme
    vim.cmd 'colorscheme catppuccin-nvim'
  end,
  ---@type CatppuccinOptions
  opts = {
    compile_path = vim.fn.stdpath 'cache' .. '/catppuccin',
    flavour = 'mocha', -- latte, frappe, macchiato, mocha
    transparent_background = false,
    styles = {
      comments = { 'italic' },
      conditionals = {},
      loops = {},
      functions = { 'italic' },
      keywords = { 'italic' },
      strings = {},
      variables = {},
      numbers = {},
      booleans = {},
      properties = {},
      types = { 'italic' },
      operators = {},
    },
    integrations = {
      alpha = true,
      blink_cmp = true,
      cmp = false,
      fidget = false,
      gitsigns = true,
      native_lsp = {
        enabled = true,
        virtual_text = {
          errors = { 'italic' },
          hints = { 'italic' },
          warnings = { 'italic' },
          information = { 'italic' },
        },
        underlines = {
          errors = { 'underline' },
          hints = { 'underline' },
          warnings = { 'underline' },
          information = { 'underline' },
        },
        inlay_hints = {
          background = true,
        },
      },
      neotest = true,
      neotree = true,
      fzf = true,
      treesitter = true,
    },
    highlight_overrides = {
      mocha = function(mocha)
        return {
          NormalFloat = { bg = mocha.none },

          ['@stars.org'] = { fg = mocha.peach },
          ['@org.headline.level1'] = { fg = mocha.blue }, -- Headline at level 1 - linked to Title
          ['@org.headline.level2'] = { fg = mocha.blue }, -- Headline at level 2 - linked to Constant
          ['@org.headline.level3'] = { fg = mocha.blue }, -- Headline at level 3 - linked to Identifier
          ['@org.headline.level4'] = { fg = mocha.blue }, -- Headline at level 4 - linked to Statement
          ['@org.headline.level5'] = { fg = mocha.blue }, -- Headline at level 5 - linked to PreProc
          ['@org.headline.level6'] = { fg = mocha.blue }, -- Headline at level 6 - linked to Type
          ['@org.headline.level7'] = { fg = mocha.blue }, -- Headline at level 7 - linked to Special
          ['@org.headline.level8'] = { fg = mocha.blue }, -- Headline at level 8 - linked to String
          -- ['@org.priority.highest'] = {}, -- Highest priority marker - linked to @comment.error
          -- ['@org.priority.high'] = {}, -- High priority marker - Not linked to anything, defaults to normal text
          -- ['@org.priority.default'] = {}, -- Default priority marker - Not linked to anything, defaults to normal text
          -- ['@org.priority.low'] = {}, -- Lowest priority marker - Not linked to anything, defaults to normal text
          -- ['@org.priority.lowest'] = {}, -- Lowest priority marker - Not linked to anything, defaults to normal text
          -- ['@org.timestamp.active'] = {}, -- An active timestamp - linked to @keyword
          -- ['@org.timestamp.inactive'] = {}, -- An inactive timestamp - linked to @comment
          ['@org.keyword.todo'] = { fg = mocha.red }, -- TODO keywords color - Parsed from Error (see note below)
          ['@org.keyword.done'] = { fg = mocha.green }, -- DONE keywords color - Parsed from DiffAdd (see note below)
          ['@org.bullet'] = { fg = mocha.lavender }, -- A normal bullet under a header item - linked to @markup.list
          -- ['@org.properties'] = {}, -- Property drawer start/end delimiters - linked to @property
          -- ['@org.drawer'] = {}, -- Drawer start/end delimiters - linked to @property
          -- ['@org.tag'] = {}, -- A tag for a headline item, shown on the righthand side like :foo - linked to @tag.attribute
          -- ['@org.plan'] = {}, -- SCHEDULED, DEADLINE, CLOSED, etc. keywords - linked to Constant
          -- ['@org.block'] = {}, -- A begin/end block (example#begin_src) - linked to @comment
          -- ['@org.inline_block'] = {}, -- A src_lang block (example print('foo') ) - linked to @comment
          -- ['@org.comment'] = {}, -- A comment block - linked to @comment
          -- ['@org.latex_env'] = {}, -- LaTeX block - linked to @markup.environment
          -- ['@org.directive'] = {}, -- Blocks starting with #+ - linked to @comment
          -- ['@org.checkbox'] = {}, -- The default checkbox highlight, including square brackets - linked to @markup.list.unchecked
          -- ['@org.checkbox.halfchecked'] = {}, -- A checkbox status (marker between []) checked with [-] - linked to @markup.list.unchecked
          -- ['@org.checkbox.checked'] = {}, -- A checkbox status (marker between []) checked with either [x] or [X] - linked to @markup.list.checked
          ['@org.bold'] = { fg = mocha.text, style = { 'bold' } }, -- bold text - linked to @markup.strong,
          ['@org.bold.delimiter'] = { fg = mocha.text, style = { 'bold' } }, -- bold text delimiter * - linked to @markup.strong,
          ['@org.italic'] = { fg = mocha.text, style = { 'italic' } }, -- italic text - linked to @markup.italic,
          ['@org.italic.delimiter'] = { fg = mocha.text, style = { 'italic' } }, -- italic text delimiter / - linked to @markup.italic,
          ['@org.strikethrough'] = { fg = mocha.text, style = { 'strikethrough' } }, -- strikethrough text - linked to @markup.strikethrough,
          ['@org.strikethrough.delimiter'] = { fg = mocha.text, style = { 'strikethrough' } }, -- strikethrough text delimiter + - linked to @markup.strikethrough,
          ['@org.underline'] = { fg = mocha.text, style = { 'underline' } }, -- underline text - linked to @markup.underline,
          ['@org.underline.delimiter'] = { fg = mocha.text, style = { 'underline' } }, -- underline text delimiter _ - linked to @markup.underline,
          ['@org.code'] = { fg = mocha.green, style = { 'altfont' } }, -- code text - linked to @markup.raw,
          ['@org.code.delimiter'] = { fg = mocha.green, style = { 'altfont' } }, -- code text delimiter ~ - linked to @markup.raw,
          ['@org.verbatim'] = { fg = mocha.green, style = { 'altfont' } }, -- verbatim text - linked to @markup.raw,
          ['@org.verbatim.delimiter'] = { fg = mocha.green, style = { 'altfont' } }, -- verbatim text delimiter = - linked to @markup.raw,
          ['@org.hyperlink'] = { fg = mocha.blue, style = { 'underline', 'italic' } }, -- link without description (example [[file:/this/is/link]]) - linked to @markup.link,
          ['@org.hyperlink.url'] = { fg = mocha.blue, style = { 'underline', 'italic' } }, -- Url part of the link with description (example [[file:/this/is/url][description]]) - linked to @markup.link.url,
          ['@org.hyperlink.desc'] = { fg = mocha.blue, style = { 'underline', 'italic' } }, -- Description part of the link with description (example [[file:/path/to/file][description]]) - linked to @markup.link.label,
          -- ['@org.latex'] = {}, -- Inline latex - linked to @markup.math,
          -- ['@org.table.delimiter'] = {}, -- - | and - delimiters in tables - linked to @punctuation.special,
          -- ['@org.table.heading'] = {}, -- - Table headings - linked to @markup.heading,
          -- ['@org.edit_src'] = {}, -- - The highlight for the source content in an Org buffer while it is being edited in an edit special buffer - linked to Visual,
          ['@org.agenda.deadline'] = { fg = mocha.text }, -- A item deadline in the agenda view - Parsed from Error (see note below)
          -- ['@org.agenda.deadline.upcoming'] = {}, -- A item upcoming deadline in the agenda view - Parsed from WarningMsg (see note below)
          ['@org.agenda.scheduled'] = { fg = mocha.text }, -- A scheduled item in the agenda view - Parsed from DiffAdd (see note below)
          -- ['@org.agenda.scheduled_past'] = {}, -- A item past its scheduled date in the agenda view - Parsed from WarningMsg (see note below)
          -- ['@org.agenda.time_grid'] = {}, -- Time grid line - Parsed from WarningMsg (see note below)
          ['@org.agenda.day'] = { fg = mocha.blue }, -- Highlight for all days in Agenda view - linked to Statement
          ['@org.agenda.today'] = { fg = mocha.blue, style = { 'underline' } }, -- Highlight for today in Agenda view - linked to @org.bold
          ['@org.agenda.weekend'] = { fg = mocha.blue, style = { 'italic' } }, -- Highlight for weekend days in Agenda view - linked to @org.bold
          -- ['@org.agenda.weekend.today'] = {}, -- Highlight for today when it is a weekend day in Agenda view - linked to @org.bold
          -- ['@org.footnote'] = {}, -- Highlight for footnote definition - linked to @markup.link.url
          -- ['@org.footnote.reference'] = {}, -- Highlight for footnote reference - linked to @markup.link.url
        }
      end,
    },
  },
}
