---@type LazySpec
return {
  --- https://github.com/yetone/avante.nvim
  'yetone/avante.nvim',
  event = 'VeryLazy',
  build = 'make',
  ---@class avante.Config
  opts = {
    debug = false,
    provider = 'claude',
    openai = {
      endpoint = 'https://api.openai.com/v1',
      model = 'gpt-4o',
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 4096,
      ['local'] = false,
    },
    copilot = {
      endpoint = 'https://api.githubcopilot.com',
      model = 'gpt-4o-2024-05-13',
      proxy = nil, -- [protocol://]host[:port] Use this proxy
      allow_insecure = false, -- Allow insecure server connections
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 4096,
    },
    azure = {
      endpoint = '', -- example: "https://<your-resource-name>.openai.azure.com"
      deployment = '', -- Azure deployment name (e.g., "gpt-4o", "my-gpt-4o-deployment")
      api_version = '2024-06-01',
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 4096,
      ['local'] = false,
    },
    claude = {
      endpoint = 'https://api.anthropic.com',
      model = 'claude-3-5-sonnet-20240620',
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 4096,
      ['local'] = false,
    },
    gemini = {
      endpoint = 'https://generativelanguage.googleapis.com/v1beta/models',
      model = 'gemini-1.5-pro',
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 4096,
      ['local'] = false,
    },
    cohere = {
      endpoint = 'https://api.cohere.com/v1',
      model = 'command-r-plus',
      timeout = 30000, -- Timeout in milliseconds
      temperature = 0,
      max_tokens = 3072,
      ['local'] = false,
    },
    ---To add support for custom provider, follow the format below
    ---See https://github.com/yetone/avante.nvim/README.md#custom-providers for more details
    vendors = {},
    ---Specify the behaviour of avante.nvim
    ---1. auto_apply_diff_after_generation: Whether to automatically apply diff after LLM response.
    ---                                     This would simulate similar behaviour to cursor. Default to false.
    ---2. auto_set_highlight_group: Whether to automatically set the highlight group for the current line. Default to true.
    behaviour = {
      auto_set_highlight_group = true,
      auto_apply_diff_after_generation = false,
    },
    history = {
      storage_path = vim.fn.stdpath 'state' .. '/avante',
    },
    highlights = {
      diff = {
        current = 'DiffText',
        incoming = 'DiffAdd',
      },
    },
    mappings = {
      ask = '<leader>aa',
      edit = '<leader>ae',
      refresh = '<leader>ar',
      diff = {
        ours = 'co',
        theirs = 'ct',
        none = 'c0',
        both = 'cb',
        next = ']x',
        prev = '[x',
      },
      jump = {
        next = ']]',
        prev = '[[',
      },
      submit = {
        normal = '<CR>',
        insert = '<C-s>',
      },
      toggle = {
        debug = '<leader>ad',
        hint = '<leader>ah',
      },
    },
    windows = {
      wrap = true, -- similar to vim.o.wrap
      width = 30, -- default % based on available width
      sidebar_header = {
        align = 'center', -- left, center, right for title
        rounded = true,
      },
    },
    diff = {
      autojump = true,
      list_opener = 'copen',
    },
    hints = {
      enabled = true,
    },
  },
  dependencies = {
    'nvim-tree/nvim-web-devicons',
    'stevearc/dressing.nvim',
    'nvim-lua/plenary.nvim',
    'MunifTanjim/nui.nvim',
    --- The below is optional, make sure to setup it properly if you have lazy=true
    -- {
    --   'MeanderingProgrammer/render-markdown.nvim',
    --   opts = {
    --     file_types = { 'markdown', 'Avante' },
    --   },
    --   ft = { 'markdown', 'Avante' },
    -- },
  },
}
