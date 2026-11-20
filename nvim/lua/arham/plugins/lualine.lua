return {
  "nvim-lualine/lualine.nvim",
  dependencies = { "nvim-tree/nvim-web-devicons" },
  config = function()
    local lualine = require("lualine")
    local lazy_status = require("lazy.status")
    
    local colors = {
      accent = "#b7d4ed",
      slate = "#758a9b",
      silver = "#b2bcc4",
      warn = "#e5c07b",
      crit = "#e06c75",
      fg = "#d8dadd",
      muted = "#61768f",
      bg = "NONE",
      surface = "#1d222b",
    }

    local my_lualine_theme = {
      normal = {
        a = { bg = colors.accent, fg = "#0b0b0c", gui = "bold" },
        b = { bg = colors.surface, fg = colors.fg },
        c = { bg = colors.bg, fg = colors.silver },
      },
      insert = {
        a = { bg = colors.slate, fg = "#0b0b0c", gui = "bold" },
        b = { bg = colors.surface, fg = colors.fg },
        c = { bg = colors.bg, fg = colors.silver },
      },
      visual = {
        a = { bg = colors.silver, fg = "#0b0b0c", gui = "bold" },
        b = { bg = colors.surface, fg = colors.fg },
        c = { bg = colors.bg, fg = colors.silver },
      },
      command = {
        a = { bg = colors.warn, fg = "#0b0b0c", gui = "bold" },
        b = { bg = colors.surface, fg = colors.fg },
        c = { bg = colors.bg, fg = colors.silver },
      },
      replace = {
        a = { bg = colors.crit, fg = "#0b0b0c", gui = "bold" },
        b = { bg = colors.surface, fg = colors.fg },
        c = { bg = colors.bg, fg = colors.silver },
      },
      inactive = {
        a = { bg = colors.bg, fg = colors.muted, gui = "bold" },
        b = { bg = colors.bg, fg = colors.muted },
        c = { bg = colors.bg, fg = colors.muted },
      },
    }

    -- configure lualine with modified theme
    lualine.setup({
      options = {
        theme = my_lualine_theme,
        component_separators = { left = "│", right = "│" },
        section_separators = { left = "", right = "" },
      },
      sections = {
        lualine_x = {
          {
            lazy_status.updates,
            cond = lazy_status.has_updates,
            color = { fg = colors.accent },
          },
          { "encoding" },
          { "fileformat" },
          { "filetype" },
        },
      },
    })
  end,
}
