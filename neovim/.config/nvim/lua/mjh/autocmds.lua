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
  callback = function(lsp_attach_event)
    local client = vim.lsp.get_client_by_id(lsp_attach_event.data.client_id)
    if client and client.server_capabilities.documentHighlightProvider then
      local highlight_group = vim.api.nvim_create_augroup('mjh-lsp-highlight', { clear = false })
      vim.api.nvim_create_autocmd({ 'CursorHold', 'CursorHoldI' }, {
        group = highlight_group,
        buffer = lsp_attach_event.buf,
        callback = vim.lsp.buf.document_highlight,
      })

      vim.api.nvim_create_autocmd({ 'CursorMoved', 'CursorMovedI' }, {
        group = highlight_group,
        buffer = lsp_attach_event.buf,
        callback = vim.lsp.buf.clear_references,
      })

      vim.api.nvim_create_autocmd('LspDetach', {
        group = vim.api.nvim_create_augroup('mjh-lsp-detach', { clear = true }),
        callback = function(lsp_detach_event)
          vim.lsp.buf.clear_references()
          vim.api.nvim_clear_autocmds { group = 'mjh-lsp-highlight', buffer = lsp_detach_event.buf }
        end,
      })
    end
  end,
})
