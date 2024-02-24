-- Highlight on yank
vim.api.nvim_create_autocmd('TextYankPost', {
  group = vim.api.nvim_create_augroup('HighlightYank', { clear = true }),
  callback = function()
    vim.highlight.on_yank {
      higroup = 'LspReferenceText',
      timeout = 100,
    }
  end,
})

-- Resolve document highlights on CursorHold(I)
-- and clear them on CursorMoved(I)
vim.api.nvim_create_autocmd('LspAttach', {
  group = vim.api.nvim_create_augroup('mjh-lsp-attach', { clear = true }),
  callback = function(event)
    vim.api.nvim_create_autocmd({ 'CursorHold', 'CursorHoldI' }, {
      buffer = event.buf,
      callback = vim.lsp.buf.document_highlight,
    })

    vim.api.nvim_create_autocmd({ 'CursorMoved', 'CursorMovedI' }, {
      buffer = event.buf,
      callback = vim.lsp.buf.clear_references,
    })
  end,
})
