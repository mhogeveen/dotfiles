local status_cmp, cmp = pcall(require, "cmp")
if not status_cmp then
  print("CMP is not installed")
  return
end

local status_luasnip, luasnip = pcall(require, "luasnip")
if not status_luasnip then
  print("Luasnip is not installed")
  return
end

require("luasnip.loaders.from_vscode").lazy_load()

local status_lspkind, lspkind = pcall(require, "lspkind")
if not status_lspkind then
  print("Lspkind is not installed")
  return
end

lspkind.init()

cmp.setup({
  snippet = {
    expand = function(args)
      luasnip.lsp_expand(args.body)
    end,
  },
  mapping = {
    ["<C-n>"] = cmp.mapping.select_next_item(),
    ["<C-p>"] = cmp.mapping.select_prev_item(),
    ["<C-b>"] = cmp.mapping.scroll_docs(-2),
    ["<C-f>"] = cmp.mapping.scroll_docs(2),
    ["<CR>"] = cmp.mapping.confirm({
      select = false,
      behavior = cmp.ConfirmBehavior.Insert,
    }),
  },
  sources = cmp.config.sources({
    { name = "nvim_lsp" },
    { name = "nvim_lua" },
    { name = "cmp_tabnine" },
    { name = "luasnip" },
    { name = "path" },
    { name = "buffer",     keyword_length = 5 },
  }),
  formatting = {
    format = lspkind.cmp_format({
      menu = {
        nvim_lsp = "[LSP]",
        nvim_lua = "[API]",
        cmp_tabnine = "[TN]",
        luasnip = "[Snip]",
        path = "[Path]",
        buffer = "[Buf]",
      },
    }),
  },
  window = {
    completion = cmp.config.window.bordered({
      border = "single",
      winhighlight = "Normal:Normal,FloatBorder:TelescopeBorder,CursorLine:Visual,Search:None",
    }),
    documentation = cmp.config.window.bordered({
      border = "single",
      winhighlight = "Normal:Normal,FloatBorder:TelescopeBorder,CursorLine:Visual,Search:None",
    }),
  },
})
