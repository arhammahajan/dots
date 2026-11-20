return {
  "folke/tokyonight.nvim",
  priority = 1000,
  config = function()
    local transparent = true -- blends with terminal opacity (kitty 0.55)

    -- Obsidian & muted slate-blue palette matching niri / kitty / fuzzel / quickshell / yazi
    local bg = "#0b0b0c"
    local bg_dark = "#0b0b0c"
    local bg_float = "#121418"
    local bg_highlight = "#1d222b"
    local bg_search = "#2e3c4a"
    local bg_visual = "#26313d"
    local fg = "#d8dadd"
    local fg_dark = "#b2bcc4"
    local fg_muted = "#758a9b"
    local fg_gutter = "#61768f"
    local border = "#2a303c"
    local border_focus = "#61768f"
    local accent = "#b7d4ed"
    local warn = "#e5c07b"
    local crit = "#e06c75"

    require("tokyonight").setup({
      style = "night",
      transparent = transparent,
      styles = {
        sidebars = transparent and "transparent" or "dark",
        floats = transparent and "transparent" or "dark",
      },
      on_colors = function(colors)
        colors.bg = bg
        colors.bg_dark = transparent and colors.none or bg_dark
        colors.bg_float = transparent and colors.none or bg_float
        colors.bg_highlight = bg_highlight
        colors.bg_popup = bg_float
        colors.bg_search = bg_search
        colors.bg_sidebar = transparent and colors.none or bg_dark
        colors.bg_statusline = transparent and colors.none or bg_dark
        colors.bg_visual = bg_visual
        colors.border = border
        colors.border_highlight = border_focus
        colors.fg = fg
        colors.fg_dark = fg_dark
        colors.fg_float = fg
        colors.fg_gutter = fg_gutter
        colors.fg_sidebar = fg_dark
        colors.comment = fg_gutter
        colors.warning = warn
        colors.error = crit
        colors.info = accent
        colors.hint = fg_muted
      end,
      on_highlights = function(hl, c)
        local float_bg = transparent and c.none or bg_float

        -- Core UI & transparency
        hl.CursorLine = { bg = bg_highlight }
        hl.CursorColumn = { bg = bg_highlight }
        hl.LineNr = { fg = fg_gutter }
        hl.CursorLineNr = { fg = accent, bold = true }
        hl.SignColumn = { bg = c.none }
        hl.WinSeparator = { fg = border, bg = c.none }
        hl.VertSplit = { fg = border, bg = c.none }
        hl.FloatBorder = { fg = border_focus, bg = float_bg }
        hl.FloatTitle = { fg = accent, bg = float_bg, bold = true }
        hl.NormalFloat = { fg = fg, bg = float_bg }

        -- Search & selection (matching fuzzel & yazi)
        hl.Visual = { bg = bg_visual }
        hl.Search = { fg = fg, bg = bg_search }
        hl.IncSearch = { fg = bg, bg = accent, bold = true }
        hl.CurSearch = { fg = bg, bg = accent, bold = true }
        hl.MatchParen = { fg = accent, bg = bg_highlight, bold = true }

        -- Popup menu / cmp
        hl.Pmenu = { fg = fg_dark, bg = bg_float }
        hl.PmenuSel = { fg = fg, bg = bg_highlight, bold = true }
        hl.PmenuSbar = { bg = bg_float }
        hl.PmenuThumb = { bg = fg_gutter }

        -- Telescope
        hl.TelescopeNormal = { fg = fg, bg = float_bg }
        hl.TelescopeBorder = { fg = border_focus, bg = float_bg }
        hl.TelescopeTitle = { fg = accent, bg = float_bg, bold = true }
        hl.TelescopeSelection = { fg = fg, bg = bg_highlight, bold = true }
        hl.TelescopeMatching = { fg = accent, bold = true }

        -- NvimTree
        hl.NvimTreeNormal = { fg = fg_dark, bg = c.none }
        hl.NvimTreeNormalNC = { fg = fg_dark, bg = c.none }
        hl.NvimTreeWinSeparator = { fg = border, bg = c.none }
        hl.NvimTreeFolderName = { fg = accent }
        hl.NvimTreeOpenedFolderName = { fg = accent, bold = true }
        hl.NvimTreeFolderIcon = { fg = fg_muted }
        hl.NvimTreeIndentMarker = { fg = border }
        hl.NvimTreeCursorLine = { bg = bg_highlight }

        -- Alpha dashboard & indent guides
        hl.AlphaHeader = { fg = accent }
        hl.AlphaButtons = { fg = fg_dark }
        hl.AlphaShortcut = { fg = accent, bold = true }
        hl.AlphaFooter = { fg = fg_muted, italic = true }
        hl.IblIndent = { fg = border }
        hl.IblScope = { fg = fg_gutter }

        -- Diagnostics & titles
        hl.Title = { fg = accent, bold = true }
        hl.Directory = { fg = accent }
        hl.DiagnosticError = { fg = crit }
        hl.DiagnosticWarn = { fg = warn }
        hl.DiagnosticInfo = { fg = accent }
        hl.DiagnosticHint = { fg = fg_muted }
      end,
    })

    vim.cmd("colorscheme tokyonight")
  end,
}
