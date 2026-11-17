use crate::app::{
    App, ConfirmAction, FormField, Mode, NewSpaceForm, RenameForm, Row, RowKind, TextField,
};
use crate::canvas::Screen;
use crate::layout;
use crate::model::{now_ms, AttentionFilter, Status};
use crate::theme::{self, Palette, Style};
use crate::util;

#[derive(Clone)]
struct Span {
    text: String,
    style: Style,
}

impl Span {
    fn new(text: impl Into<String>, style: Style) -> Self {
        Span {
            text: text.into(),
            style,
        }
    }
}

fn spans_width(spans: &[Span]) -> usize {
    spans.iter().map(|span| util::width(&span.text)).sum()
}

fn draw_spans(screen: &mut Screen, x: usize, y: usize, spans: &[Span], right: usize) -> usize {
    let mut cursor = x;
    for span in spans {
        if cursor >= right {
            break;
        }
        let text = util::truncate(&span.text, right - cursor);
        let width = util::width(&text);
        screen.put(cursor, y, &text, span.style);
        cursor += width;
    }
    cursor
}

fn draw_spans_right(screen: &mut Screen, right: usize, y: usize, spans: &[Span]) {
    let width = spans_width(spans);
    draw_spans(screen, right.saturating_sub(width), y, spans, right);
}

fn key_span(text: &str, bg: theme::Rgb) -> Span {
    Span::new(text, Style::new(Palette::ACCENT, bg).bold())
}

fn label_span(text: &str, bg: theme::Rgb) -> Span {
    Span::new(text, Style::new(Palette::SUBTEXT, bg).dim())
}

fn separator_span(bg: theme::Rgb) -> Span {
    Span::new("  ", Style::new(Palette::MUTED, bg))
}

pub fn render(app: &App, rows: usize, cols: usize) -> String {
    draw(app, rows, cols).to_ansi()
}

#[cfg(test)]
pub fn render_lines(app: &App, rows: usize, cols: usize) -> Vec<String> {
    draw(app, rows, cols).to_lines()
}

fn draw(app: &App, rows: usize, cols: usize) -> Screen {
    let mut screen = Screen::new(cols, rows, Style::new(Palette::TEXT, Palette::PANEL));
    if rows < 4 || cols < 16 {
        screen.fill(Style::new(Palette::TEXT, Palette::PANEL));
        screen.put(
            0,
            0,
            "Spaces",
            Style::new(Palette::ACCENT, Palette::PANEL).bold(),
        );
        return screen;
    }
    let geometry = layout::geometry(rows, cols);
    let right_title = format!(
        "{} space{} · {} working{}",
        app.spaces.len(),
        if app.spaces.len() == 1 { "" } else { "s" },
        app.spaces
            .iter()
            .map(|space| space.counts.working)
            .sum::<usize>(),
        if app
            .spaces
            .iter()
            .map(|space| space.counts.attention_total())
            .sum::<usize>()
            > 0
        {
            format!(
                " · {} blocked",
                app.spaces
                    .iter()
                    .map(|space| space.counts.attention_total())
                    .sum::<usize>()
            )
        } else {
            String::new()
        }
    );
    screen.draw_box(
        0,
        0,
        cols,
        rows,
        Style::new(Palette::BORDER, Palette::PANEL),
        Style::new(Palette::TEXT, Palette::PANEL),
        Some((
            " Spaces ",
            Style::new(Palette::ACCENT, Palette::PANEL).bold(),
        )),
        Some((
            &right_title,
            Style::new(Palette::SUBTEXT, Palette::PANEL).dim(),
        )),
    );
    if let Some(search_y) = geometry.search_y {
        draw_search_line(&mut screen, app, search_y, cols);
        draw_rule(&mut screen, search_y + 1, cols);
    }
    draw_list(&mut screen, app, &geometry);
    if let Some(detail_y) = geometry.detail_y {
        draw_rule(&mut screen, detail_y.saturating_sub(1), cols);
        draw_detail_line(&mut screen, app, detail_y, cols);
    } else {
        draw_rule(&mut screen, geometry.hints_y.saturating_sub(1), cols);
    }
    draw_hints(&mut screen, app, geometry.hints_y, cols);
    match &app.mode {
        Mode::NewSpace(form) => draw_new_space_dialog(&mut screen, form, rows, cols),
        Mode::Rename(form) => draw_rename_dialog(&mut screen, form, rows, cols),
        Mode::Confirm(action) => draw_confirm_dialog(&mut screen, action, rows, cols),
        Mode::Help => draw_help(&mut screen, rows, cols),
        Mode::HookDetail { scroll, loading } => {
            draw_hook_detail(&mut screen, app, *scroll, *loading, rows, cols)
        }
        _ => {}
    }
    screen
}

fn draw_search_line(screen: &mut Screen, app: &App, y: usize, cols: usize) {
    let bg = Palette::HEADER;
    screen.fill_rect(
        1,
        y,
        cols.saturating_sub(2),
        1,
        ' ',
        Style::new(Palette::TEXT, bg),
    );
    screen.fill_rect(
        2,
        y,
        3,
        1,
        ' ',
        Style::new(Palette::ACCENT, Palette::SURFACE0).bold(),
    );
    screen.put(
        3,
        y,
        "/",
        Style::new(Palette::ACCENT, Palette::SURFACE0).bold(),
    );
    let filter_width = filter_spans(app, cols)
        .map(|spans| spans_width(&spans) + 2)
        .unwrap_or(0);
    let right = cols.saturating_sub(1 + filter_width);
    let text_x = 6;
    if matches!(app.mode, Mode::Search) {
        let chars = app.search.chars();
        let cursor = app.search.cursor();
        let prefix: String = chars.iter().take(cursor).collect();
        let prefix = util::truncate(&prefix, right.saturating_sub(text_x + 1));
        screen.put(text_x, y, &prefix, Style::new(Palette::TEXT, bg));
        let caret_x = text_x + util::width(&prefix);
        if caret_x < right {
            screen.put(caret_x, y, "_", Style::new(Palette::ACCENT, bg).bold());
            let rest: String = chars.iter().skip(cursor).collect();
            screen.put(
                caret_x + 1,
                y,
                &util::truncate(&rest, right.saturating_sub(caret_x + 1)),
                Style::new(Palette::TEXT, bg),
            );
        }
    } else if app.search.is_empty() {
        screen.put(
            text_x,
            y,
            "Search spaces, agents, paths…",
            Style::new(Palette::MUTED, bg).dim(),
        );
    } else {
        screen.put(
            text_x,
            y,
            &util::truncate(&app.search.value(), right.saturating_sub(text_x)),
            Style::new(Palette::TEXT, bg),
        );
    }
    if let Some(chips) = filter_spans(app, cols) {
        draw_spans_right(screen, cols.saturating_sub(2), y, &chips);
    }
}

fn filter_spans(app: &App, cols: usize) -> Option<Vec<Span>> {
    if cols < 48 {
        return None;
    }
    let labels: Vec<(&str, bool)> = if cols >= 84 {
        AttentionFilter::all()
            .iter()
            .map(|filter| (filter.label(), *filter == app.active_filter()))
            .collect()
    } else {
        vec![(app.active_filter().label(), true)]
    };
    let mut spans = Vec::new();
    for (index, (label, active)) in labels.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::new(" ", Style::new(Palette::MUTED, Palette::HEADER)));
        }
        let style = if active {
            Style::new(Palette::ACCENT, Palette::SURFACE0).bold()
        } else {
            Style::new(Palette::MUTED, Palette::HEADER).dim()
        };
        spans.push(Span::new(format!(" {} ", label), style));
    }
    Some(spans)
}

fn draw_rule(screen: &mut Screen, y: usize, cols: usize) {
    if y >= screen.height() || cols < 2 {
        return;
    }
    screen.put(0, y, "├", Style::new(Palette::BORDER, Palette::PANEL));
    screen.put(
        cols - 1,
        y,
        "┤",
        Style::new(Palette::BORDER, Palette::PANEL),
    );
    screen.fill_rect(
        1,
        y,
        cols.saturating_sub(2),
        1,
        '─',
        Style::new(Palette::BORDER, Palette::PANEL),
    );
}

fn draw_list(screen: &mut Screen, app: &App, geometry: &layout::Geometry) {
    let selected = app.selected_index();
    let start = layout::window_start(selected.unwrap_or(0), app.rows.len(), geometry.list_height);
    let scrollbar = app.rows.len() > geometry.list_height && geometry.cols > 30;
    let content_right = if scrollbar {
        geometry.cols.saturating_sub(2)
    } else {
        geometry.cols.saturating_sub(1)
    };
    if app.rows.is_empty() {
        screen.put(
            3,
            geometry.list_top,
            if app.spaces.is_empty() {
                "No running spaces. Press n to create one."
            } else {
                "No spaces match this filter."
            },
            Style::new(Palette::MUTED, Palette::PANEL).dim(),
        );
    }
    for (offset, row) in app
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .take(geometry.list_height)
    {
        let y = geometry.list_top + offset.saturating_sub(start);
        let is_selected = selected == Some(offset);
        let bg = if is_selected {
            Palette::SELECTION
        } else {
            Palette::PANEL
        };
        screen.fill_rect(
            1,
            y,
            geometry.cols.saturating_sub(2),
            1,
            ' ',
            Style::new(Palette::TEXT, bg),
        );
        if is_selected {
            screen.put(1, y, "▌", Style::new(Palette::ACCENT, bg).bold());
        }
        match &row.kind {
            RowKind::Space => draw_space_row(screen, app, row, y, content_right, is_selected, bg),
            RowKind::Pane => draw_pane_row(screen, app, row, y, content_right, is_selected, bg),
            RowKind::ExitedHeader => {
                screen.put(
                    3,
                    y,
                    &format!("EXITED · {}", app.exited.len()),
                    Style::new(Palette::MUTED, bg).bold().dim(),
                );
            }
            RowKind::Exited { name } => {
                screen.put(3, y, "○", Style::new(Palette::MUTED, bg));
                screen.put(5, y, name, Style::new(Palette::SUBTEXT, bg).dim());
                if let Some(space) = app.exited.iter().find(|space| &space.name == name) {
                    screen.put_right(
                        content_right.saturating_sub(1),
                        y,
                        &util::format_ago(space.age),
                        Style::new(Palette::MUTED, bg).dim(),
                    );
                }
            }
            RowKind::Create { name } => {
                screen.put(3, y, "+", Style::new(Palette::GREEN, bg).bold());
                screen.put(
                    5,
                    y,
                    &format!("Create space “{}”", name),
                    Style::new(Palette::GREEN, bg).bold(),
                );
                if content_right > 52 {
                    if let Some(cwd) = app
                        .selected_space()
                        .and_then(|space| space.cwd.clone())
                        .or_else(|| app.current_space().and_then(|space| space.cwd.clone()))
                        .or_else(|| app.default_cwd.clone())
                    {
                        screen.put_right(
                            content_right.saturating_sub(1),
                            y,
                            &format!("in {}", util::shorten_path(&cwd, 24, app.home.as_deref())),
                            Style::new(Palette::MUTED, bg).dim(),
                        );
                    }
                }
            }
        }
    }
    if scrollbar {
        let track_x = geometry.cols.saturating_sub(2);
        for y in geometry.list_top..geometry.list_top + geometry.list_height {
            screen.put(
                track_x,
                y,
                "│",
                Style::new(Palette::SURFACE1, Palette::PANEL),
            );
        }
        let visible = geometry.list_height;
        let total = app.rows.len();
        let thumb_size = (visible * visible)
            .checked_div(total)
            .unwrap_or(0)
            .max(1)
            .min(visible);
        let max_start = total.saturating_sub(visible);
        let thumb_start = start
            .saturating_mul(visible.saturating_sub(thumb_size))
            .checked_div(max_start)
            .unwrap_or(0);
        for offset in 0..thumb_size {
            screen.put(
                track_x,
                geometry.list_top + thumb_start + offset,
                "█",
                Style::new(Palette::SUBTEXT, Palette::PANEL),
            );
        }
    }
}

fn draw_space_row(
    screen: &mut Screen,
    app: &App,
    row: &Row,
    y: usize,
    content_right: usize,
    selected: bool,
    bg: theme::Rgb,
) {
    let Some(space) = row.space_idx.and_then(|idx| app.spaces.get(idx)) else {
        return;
    };
    let marker = if space.is_current && !selected {
        "▏"
    } else {
        " "
    };
    screen.put(1, y, marker, Style::new(Palette::ACCENT, bg));
    screen.put(
        3,
        y,
        space.status.icon(),
        Style::new(theme::status_color(space.status), bg).bold(),
    );
    let name_style = if space.is_current || selected {
        Style::new(Palette::ACCENT, bg).bold()
    } else {
        Style::new(Palette::TEXT, bg).bold()
    };
    let mut x = 5;
    let name = util::truncate(&space.name, 30);
    screen.put(x, y, &name, name_style);
    x += util::width(&name);
    if space.is_current {
        let badge = " current ";
        if x + util::width(badge) < content_right {
            screen.put(
                x + 1,
                y,
                badge,
                Style::new(Palette::CRUST, Palette::ACCENT).bold(),
            );
            x += util::width(badge) + 1;
        }
    }
    let right_spans = space_right_spans(space, bg);
    let right_width = spans_width(&right_spans);
    let right_start = content_right.saturating_sub(right_width + 1);
    if let Some(cwd) = space.cwd.as_deref() {
        if right_start > x + 4 && content_right > 60 {
            let room = right_start.saturating_sub(x + 2);
            let cwd = util::shorten_path(cwd, room, app.home.as_deref());
            screen.put(x + 2, y, &cwd, Style::new(Palette::MUTED, bg).dim());
        }
    }
    draw_spans_right(screen, content_right.saturating_sub(1), y, &right_spans);
}

fn space_right_spans(space: &crate::model::Space, bg: theme::Rgb) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut add = |icon: &str, count: usize, status| {
        if count == 0 {
            return;
        }
        if !spans.is_empty() {
            spans.push(Span::new("  ", Style::new(Palette::MUTED, bg)));
        }
        spans.push(Span::new(
            icon,
            Style::new(theme::status_color(status), bg).bold(),
        ));
        spans.push(Span::new(
            format!("{}", count),
            Style::new(theme::status_color(status), bg),
        ));
    };
    add("●", space.counts.needs, Status::NeedsYou);
    add("✗", space.counts.failed, Status::Failed);
    add("◐", space.counts.working, Status::Working);
    add("✓", space.counts.done, Status::Done);
    if spans.is_empty() {
        spans.push(Span::new("○", Style::new(Palette::MUTED, bg)));
    }
    spans.push(Span::new("  ", Style::new(Palette::MUTED, bg)));
    spans.push(Span::new(
        format!(
            "{} tab{}",
            space.tab_count(),
            if space.tab_count() == 1 { "" } else { "s" }
        ),
        Style::new(Palette::SUBTEXT, bg).dim(),
    ));
    if space.connected_clients > 1 {
        spans.push(Span::new("  ", Style::new(Palette::MUTED, bg)));
        spans.push(Span::new(
            format!("{} clients", space.connected_clients),
            Style::new(Palette::MUTED, bg).dim(),
        ));
    }
    spans
}

fn draw_pane_row(
    screen: &mut Screen,
    app: &App,
    row: &Row,
    y: usize,
    content_right: usize,
    selected: bool,
    bg: theme::Rgb,
) {
    let (Some(space), Some(pane)) = (
        row.space_idx.and_then(|idx| app.spaces.get(idx)),
        row.space_idx
            .zip(row.pane_idx)
            .and_then(|(space_idx, pane_idx)| app.spaces.get(space_idx)?.panes.get(pane_idx)),
    ) else {
        return;
    };
    screen.put(2, y, "└", Style::new(Palette::SURFACE1, bg));
    screen.put(
        4,
        y,
        pane.status.icon(),
        Style::new(theme::status_color(pane.status), bg).bold(),
    );
    let mut x = 6;
    let app_name = util::truncate(&pane.app, 18);
    screen.put(
        x,
        y,
        &app_name,
        if selected {
            Style::new(Palette::TEXT, bg).bold()
        } else {
            Style::new(Palette::TEXT, bg)
        },
    );
    x += util::width(&app_name);
    let tab = util::truncate(&space.tab_name(pane.tab_position), 18);
    if util::width(&tab) > 0 {
        screen.put(x + 2, y, "·", Style::new(Palette::MUTED, bg));
        screen.put(x + 4, y, &tab, Style::new(Palette::SUBTEXT, bg).dim());
        x += 4 + util::width(&tab);
    }
    let right = content_right.saturating_sub(1);
    let age = if pane.age_ms > 0 && content_right > 56 {
        util::format_age(pane.age_ms)
    } else {
        String::new()
    };
    let age_width = util::width(&age);
    let text_right = right.saturating_sub(age_width + 2);
    let status_text = pane.status.label();
    let mut text_x = x + 2;
    if text_x < text_right {
        screen.put(
            text_x,
            y,
            status_text,
            Style::new(theme::status_color(pane.status), bg),
        );
        text_x += util::width(status_text);
        let detail = pane.display_text().trim();
        if !detail.is_empty()
            && !detail.eq_ignore_ascii_case(status_text)
            && text_x + 3 < text_right
        {
            screen.put(text_x, y, " · ", Style::new(Palette::MUTED, bg));
            text_x += 3;
            screen.put(
                text_x,
                y,
                &util::truncate(detail, text_right.saturating_sub(text_x)),
                Style::new(Palette::SUBTEXT, bg).dim(),
            );
        }
    }
    if !age.is_empty() {
        screen.put(
            right.saturating_sub(age_width),
            y,
            &age,
            Style::new(Palette::MUTED, bg).dim(),
        );
    }
}

fn draw_detail_line(screen: &mut Screen, app: &App, y: usize, cols: usize) {
    let bg = Palette::FOOTER;
    screen.fill_rect(
        1,
        y,
        cols.saturating_sub(2),
        1,
        ' ',
        Style::new(Palette::TEXT, bg),
    );
    let spans = if let Some(toast) = &app.toast {
        vec![
            Span::new(" ! ", Style::new(Palette::RED, bg).bold()),
            Span::new(toast.clone(), Style::new(Palette::RED, bg)),
        ]
    } else if let Some(row) = app.selected_row() {
        detail_spans(app, row, bg)
    } else {
        vec![Span::new(
            "No spaces yet. Press n to create one.",
            Style::new(Palette::MUTED, bg).dim(),
        )]
    };
    draw_spans(screen, 3, y, &spans, cols.saturating_sub(2));
}

fn detail_spans(app: &App, row: &Row, bg: theme::Rgb) -> Vec<Span> {
    match &row.kind {
        RowKind::Space => {
            let Some(space) = row.space_idx.and_then(|idx| app.spaces.get(idx)) else {
                return vec![];
            };
            let mut spans = Vec::new();
            if let Some(cwd) = space.cwd.as_deref() {
                spans.push(Span::new(
                    util::shorten_path(cwd, 48, app.home.as_deref()),
                    Style::new(Palette::SUBTEXT, bg),
                ));
            }
            spans.push(separator_span(bg));
            spans.push(label_span(
                &format!(
                    "{} tab{} · {} pane{}",
                    space.tab_count(),
                    if space.tab_count() == 1 { "" } else { "s" },
                    space.panes.len(),
                    if space.panes.len() == 1 { "" } else { "s" }
                ),
                bg,
            ));
            if space.connected_clients > 1 {
                spans.push(separator_span(bg));
                spans.push(label_span(
                    &format!("{} clients", space.connected_clients),
                    bg,
                ));
            }
            spans.push(separator_span(bg));
            spans.push(label_span(
                &format!("created {}", util::format_ago(space.created)),
                bg,
            ));
            if space.is_current {
                spans.push(separator_span(bg));
                spans.push(Span::new(
                    "current space",
                    Style::new(Palette::ACCENT, bg).bold(),
                ));
            }
            spans
        }
        RowKind::Pane => {
            let Some(space) = row.space_idx.and_then(|idx| app.spaces.get(idx)) else {
                return vec![];
            };
            let Some(pane) = row.pane_idx.and_then(|idx| space.panes.get(idx)) else {
                return vec![];
            };
            let mut spans = vec![
                Span::new(space.name.clone(), Style::new(Palette::ACCENT, bg).bold()),
                separator_span(bg),
                Span::new(
                    space.tab_name(pane.tab_position),
                    Style::new(Palette::SUBTEXT, bg),
                ),
                separator_span(bg),
                Span::new(
                    pane.status.label(),
                    Style::new(theme::status_color(pane.status), bg).bold(),
                ),
            ];
            if let Some(hook) = pane.hook.as_ref() {
                if let Some(kind) = hook.kind.as_deref().filter(|kind| !kind.is_empty()) {
                    spans.push(separator_span(bg));
                    spans.push(Span::new(kind, Style::new(Palette::RED, bg).bold()));
                }
                if !hook.event.is_empty() {
                    spans.push(separator_span(bg));
                    spans.push(label_span(&hook.event, bg));
                }
                if let Some(model) = hook.model.as_deref().filter(|model| !model.is_empty()) {
                    spans.push(separator_span(bg));
                    spans.push(label_span(
                        &format!("model {}", util::truncate(model, 18)),
                        bg,
                    ));
                }
                if let Some(session) = hook.session_id.as_deref().filter(|id| !id.is_empty()) {
                    spans.push(separator_span(bg));
                    spans.push(label_span(
                        &format!("session {}", util::truncate(session, 8)),
                        bg,
                    ));
                }
                if hook.child_count > 0 {
                    spans.push(separator_span(bg));
                    spans.push(label_span(
                        &format!(
                            "+{} agent{}",
                            hook.child_count,
                            if hook.child_count == 1 { "" } else { "s" }
                        ),
                        bg,
                    ));
                }
            }
            if let Some(cwd) = pane.cwd.as_deref() {
                spans.push(separator_span(bg));
                spans.push(label_span(
                    &util::shorten_path(cwd, 40, app.home.as_deref()),
                    bg,
                ));
            }
            if !pane.display_text().trim().is_empty() {
                spans.push(separator_span(bg));
                spans.push(Span::new(
                    pane.display_text().to_owned(),
                    Style::new(Palette::SUBTEXT, bg),
                ));
            }
            spans
        }
        RowKind::Exited { name } => {
            let saved = app
                .exited
                .iter()
                .find(|space| &space.name == name)
                .map(|space| util::format_ago(space.age))
                .unwrap_or_else(|| "unknown".to_owned());
            vec![
                Span::new("Saved space", Style::new(Palette::SUBTEXT, bg)),
                separator_span(bg),
                Span::new(
                    format!("saved {}", saved),
                    Style::new(Palette::MUTED, bg).dim(),
                ),
                separator_span(bg),
                Span::new("Enter restores it", Style::new(Palette::MUTED, bg).dim()),
            ]
        }
        RowKind::Create { name } => {
            let cwd = app
                .selected_space()
                .and_then(|space| space.cwd.clone())
                .or_else(|| app.current_space().and_then(|space| space.cwd.clone()))
                .or_else(|| app.default_cwd.clone());
            let mut spans = vec![Span::new(
                format!("Create space “{}”", name),
                Style::new(Palette::GREEN, bg).bold(),
            )];
            if let Some(cwd) = cwd {
                spans.push(separator_span(bg));
                spans.push(label_span(
                    &format!("in {}", util::shorten_path(&cwd, 52, app.home.as_deref())),
                    bg,
                ));
            }
            spans
        }
        RowKind::ExitedHeader => vec![Span::new(
            "Saved spaces from earlier Zellij sessions",
            Style::new(Palette::MUTED, bg).dim(),
        )],
    }
}

fn draw_hints(screen: &mut Screen, app: &App, y: usize, cols: usize) {
    let bg = Palette::FOOTER;
    screen.fill_rect(
        1,
        y,
        cols.saturating_sub(2),
        1,
        ' ',
        Style::new(Palette::TEXT, bg),
    );
    let narrow = cols < 72;
    let hints: Vec<(&str, &str)> = match &app.mode {
        Mode::Search => vec![
            ("type", "filter"),
            ("↑↓", "move"),
            ("Enter", "open"),
            ("Esc", "clear"),
        ],
        Mode::Help => vec![("Esc", "close help")],
        Mode::HookDetail { .. } => vec![
            ("↑↓", "scroll"),
            ("PgUp/PgDn", "page"),
            ("H", "close"),
            ("Esc", "close"),
        ],
        Mode::NewSpace(_) | Mode::Rename(_) => {
            vec![("Tab", "next field"), ("Enter", "save"), ("Esc", "cancel")]
        }
        Mode::Confirm(_) => vec![("Enter", "confirm"), ("Esc", "cancel")],
        Mode::Navigate if narrow => vec![
            ("↑↓", "move"),
            ("Enter", "open"),
            ("/", "search"),
            ("n", "new"),
            ("Esc", "close"),
        ],
        Mode::Navigate => vec![
            ("↑↓", "move"),
            ("←→", "space"),
            ("Enter", "open"),
            ("/", "search"),
            ("n", "new"),
            ("r", "rename"),
            ("x", "close"),
            ("b/w/i/d", "filter"),
            ("H", "hooks"),
            ("?", "help"),
            ("Esc", "close"),
        ],
    };
    let mut spans = Vec::new();
    for (index, (key, label)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::new(" · ", Style::new(Palette::MUTED, bg)));
        }
        spans.push(key_span(key, bg));
        spans.push(Span::new(" ", Style::new(Palette::TEXT, bg)));
        spans.push(label_span(label, bg));
    }
    draw_spans(screen, 3, y, &spans, cols.saturating_sub(2));
}

fn draw_new_space_dialog(screen: &mut Screen, form: &NewSpaceForm, rows: usize, cols: usize) {
    let width = cols.saturating_sub(4).clamp(30, 64);
    let height = rows.saturating_sub(2).clamp(7, 10);
    let (x, y) = centered(rows, cols, width, height);
    draw_dialog_box(screen, x, y, width, height, " New space ");
    let field_width = width.saturating_sub(12);
    draw_form_row(
        screen,
        x + 2,
        y + 2,
        field_width,
        "Name",
        &form.name,
        form.field == FormField::Name,
    );
    draw_form_row(
        screen,
        x + 2,
        y + 3,
        field_width,
        "Folder",
        &form.cwd,
        form.field == FormField::Cwd,
    );
    if let Some(error) = &form.error {
        screen.put(
            x + 2,
            y + 5,
            &util::truncate(error, width.saturating_sub(4)),
            Style::new(Palette::RED, Palette::BASE).bold(),
        );
    } else {
        screen.put(
            x + 2,
            y + 5,
            "The folder is the starting directory.",
            Style::new(Palette::MUTED, Palette::BASE).dim(),
        );
    }
    screen.put(
        x + 2,
        y + height.saturating_sub(2),
        "Tab next · Enter create · Esc cancel",
        Style::new(Palette::SUBTEXT, Palette::BASE).dim(),
    );
}

fn draw_rename_dialog(screen: &mut Screen, form: &RenameForm, rows: usize, cols: usize) {
    let width = cols.saturating_sub(4).clamp(28, 58);
    let height = rows.saturating_sub(2).clamp(5, 7);
    let (x, y) = centered(rows, cols, width, height);
    draw_dialog_box(screen, x, y, width, height, " Rename space ");
    draw_form_row(
        screen,
        x + 2,
        y + 2,
        width.saturating_sub(12),
        "Name",
        &form.name,
        true,
    );
    if let Some(error) = &form.error {
        screen.put(
            x + 2,
            y + 4,
            &util::truncate(error, width.saturating_sub(4)),
            Style::new(Palette::RED, Palette::BASE).bold(),
        );
    }
    screen.put(
        x + 2,
        y + height.saturating_sub(2),
        "Enter save · Esc cancel",
        Style::new(Palette::SUBTEXT, Palette::BASE).dim(),
    );
}

fn draw_confirm_dialog(screen: &mut Screen, action: &ConfirmAction, rows: usize, cols: usize) {
    let (title, message) = match action {
        ConfirmAction::KillSpace { name, tabs, panes } => (
            " Close space? ",
            format!(
                "Close “{}”? Its {} tab{} and {} pane{} will stop. This cannot be undone.",
                name,
                tabs,
                if *tabs == 1 { "" } else { "s" },
                panes,
                if *panes == 1 { "" } else { "s" },
            ),
        ),
        ConfirmAction::DeleteExited { name } => (
            " Delete saved space? ",
            format!(
                "Delete the saved state for “{}”? The space is not running, but its layout and scrollback will be removed.",
                name
            ),
        ),
    };
    let width = cols.saturating_sub(4).clamp(30, 62);
    let height = rows.saturating_sub(2).clamp(6, 9);
    let (x, y) = centered(rows, cols, width, height);
    draw_dialog_box(screen, x, y, width, height, title);
    let lines = util::wrap(&message, width.saturating_sub(4));
    for (index, line) in lines.iter().take(height.saturating_sub(4)).enumerate() {
        screen.put(
            x + 2,
            y + 2 + index,
            line,
            Style::new(Palette::TEXT, Palette::BASE),
        );
    }
    screen.put(
        x + 2,
        y + height.saturating_sub(2),
        "Enter confirm · Esc cancel",
        Style::new(Palette::SUBTEXT, Palette::BASE).dim(),
    );
}

fn push_wrapped(lines: &mut Vec<(String, Style)>, text: &str, width: usize, style: Style) {
    for line in util::wrap(text, width) {
        lines.push((line, style));
    }
}

fn draw_hook_detail(
    screen: &mut Screen,
    app: &App,
    scroll: usize,
    loading: bool,
    rows: usize,
    cols: usize,
) {
    let width = cols.saturating_sub(4).clamp(40, 96);
    let height = rows.saturating_sub(4).clamp(10, 40);
    let (x, y) = centered(rows, cols, width, height);
    draw_dialog_box(screen, x, y, width, height, " Spaces · agent hooks ");
    let Some(key) = app.hook_detail_target() else {
        return;
    };
    let summary = app.hook_summary_for(&key);
    let detail = app.hook_detail_for(&key);
    let wrap_width = width.saturating_sub(6).max(8);
    let heading = Style::new(Palette::ACCENT, Palette::BASE).bold();
    let value = Style::new(Palette::SUBTEXT, Palette::BASE);
    let muted = Style::new(Palette::MUTED, Palette::BASE).dim();
    let now = now_ms();
    let mut lines: Vec<(String, Style)> = Vec::new();

    if let Some(hook) = summary {
        let since = if hook.state_since_ms > 0 {
            hook.state_since_ms
        } else {
            hook.ts_ms
        };
        let age = now.saturating_sub(since);
        let mut header = if hook.agent.is_empty() || hook.agent == hook.app {
            format!("{} · {}", hook.app, hook.state)
        } else {
            format!("{} ({}) · {}", hook.app, hook.agent, hook.state)
        };
        if let Some(kind) = hook.kind.as_deref().filter(|kind| !kind.is_empty()) {
            header.push_str(" · ");
            header.push_str(kind);
        }
        if !hook.event.is_empty() {
            header.push_str(" · ");
            header.push_str(&hook.event);
        }
        header.push_str(" · ");
        header.push_str(&util::format_age(age));
        if hook.turn_started_ms > 0 {
            header.push_str(" · turn ");
            header.push_str(&util::format_age(now.saturating_sub(hook.turn_started_ms)));
        }
        lines.push((header, Style::new(Palette::TEXT, Palette::BASE).bold()));
        lines.push((String::new(), value));
        if let Some(model) = hook.model.as_deref().filter(|model| !model.is_empty()) {
            lines.push((format!("model    {}", model), value));
        }
        if let Some(session) = hook.session_id.as_deref().filter(|id| !id.is_empty()) {
            lines.push((format!("session  {}", session), value));
        }
        if let Some(cwd) = hook.cwd.as_deref().filter(|cwd| !cwd.is_empty()) {
            lines.push((
                format!(
                    "cwd      {}",
                    util::shorten_path(cwd, wrap_width.saturating_sub(10), app.home.as_deref())
                ),
                value,
            ));
        }
        lines.push((String::new(), value));
    }

    if let Some(detail) = detail {
        if let Some(prompt) = detail
            .prompt
            .as_deref()
            .filter(|text| !text.trim().is_empty())
        {
            lines.push(("prompt".to_owned(), heading));
            push_wrapped(&mut lines, prompt, wrap_width, value);
            lines.push((String::new(), value));
        }
        if let Some(tool) = detail.tool.as_ref() {
            lines.push((format!("tool · {}", tool.name), heading));
            if !tool.input.trim().is_empty() {
                push_wrapped(&mut lines, &tool.input, wrap_width, value);
            }
            if let Some(output) = tool
                .output
                .as_deref()
                .filter(|text| !text.trim().is_empty())
            {
                lines.push(("result".to_owned(), muted));
                push_wrapped(&mut lines, output, wrap_width, muted);
            }
            lines.push((String::new(), value));
        }
        if let Some(reply) = detail
            .reply
            .as_deref()
            .filter(|text| !text.trim().is_empty())
        {
            lines.push(("reply".to_owned(), heading));
            push_wrapped(&mut lines, reply, wrap_width, value);
            lines.push((String::new(), value));
        }
        if !detail.children.is_empty() {
            lines.push((format!("subagents ({})", detail.children.len()), heading));
            for child in &detail.children {
                let age = if child.since_ms > 0 {
                    format!("  {}", util::format_age(now.saturating_sub(child.since_ms)))
                } else {
                    String::new()
                };
                let label = format!(
                    "{}  {}  {}{}",
                    if child.kind.is_empty() {
                        "agent"
                    } else {
                        &child.kind
                    },
                    child.state,
                    child.id,
                    age
                );
                push_wrapped(&mut lines, &format!("· {}", label), wrap_width, value);
                if !child.text.trim().is_empty() {
                    push_wrapped(&mut lines, &format!("  {}", child.text), wrap_width, muted);
                }
            }
            lines.push((String::new(), value));
        }
        if detail.background > 0 {
            lines.push((format!("background tasks: {}", detail.background), muted));
        }
    } else if loading {
        lines.push(("loading hook detail…".to_owned(), muted));
    } else {
        lines.push(("No detail record for this pane.".to_owned(), muted));
    }

    let visible = height.saturating_sub(4);
    let max_scroll = lines.len().saturating_sub(visible);
    let start = scroll.min(max_scroll);
    for (offset, (text, style)) in lines.iter().skip(start).take(visible).enumerate() {
        screen.put(x + 2, y + 2 + offset, text, *style);
    }
    screen.put(
        x + 2,
        y + height.saturating_sub(2),
        "↑↓ scroll · PgUp/PgDn page · H / Esc close",
        muted,
    );
}

fn draw_help(screen: &mut Screen, rows: usize, cols: usize) {
    let width = cols.saturating_sub(4).clamp(34, 72);
    let height = rows.saturating_sub(2).clamp(8, 24);
    let (x, y) = centered(rows, cols, width, height);
    draw_dialog_box(screen, x, y, width, height, " Spaces · keys ");
    let sections: Vec<(&str, Vec<(&str, &str)>)> = vec![
        (
            "Navigate",
            vec![
                ("↑ ↓ / j k", "move between rows"),
                ("← → / h l", "previous / next space"),
                ("Enter", "open the selected space or agent"),
                ("/", "search names, agents and paths"),
                ("H", "hook details for the selected agent"),
                ("Esc", "close this navigator"),
            ],
        ),
        (
            "Spaces",
            vec![
                ("n", "new space"),
                ("r", "rename the current space"),
                ("x", "close a space"),
                ("Space", "collapse or expand"),
                ("1 … 9", "select a space"),
            ],
        ),
        (
            "Filter",
            vec![
                ("b", "blocked / needs you"),
                ("w", "working"),
                ("d", "done"),
                ("i", "idle"),
                ("a", "show everything"),
                ("Ctrl-r", "refresh"),
            ],
        ),
    ];
    let mut line = 0;
    for (heading, entries) in sections {
        if y + 2 + line >= y + height.saturating_sub(2) {
            break;
        }
        screen.put(
            x + 2,
            y + 2 + line,
            heading,
            Style::new(Palette::ACCENT, Palette::BASE).bold(),
        );
        line += 1;
        for (key, description) in entries {
            if y + 2 + line >= y + height.saturating_sub(2) {
                break;
            }
            screen.put(
                x + 3,
                y + 2 + line,
                key,
                Style::new(Palette::LAVENDER, Palette::BASE).bold(),
            );
            screen.put(
                x + 17,
                y + 2 + line,
                &util::truncate(description, width.saturating_sub(19)),
                Style::new(Palette::SUBTEXT, Palette::BASE),
            );
            line += 1;
        }
        line += 1;
    }
    screen.put(
        x + 2,
        y + height.saturating_sub(2),
        "? / Esc close",
        Style::new(Palette::MUTED, Palette::BASE).dim(),
    );
}

fn draw_dialog_box(
    screen: &mut Screen,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    title: &str,
) {
    screen.fill_rect(
        x + 1,
        y + 1,
        width,
        height,
        ' ',
        Style::new(Palette::CRUST, Palette::CRUST),
    );
    screen.draw_box(
        x,
        y,
        width,
        height,
        Style::new(Palette::ACCENT, Palette::BASE),
        Style::new(Palette::TEXT, Palette::BASE),
        Some((title, Style::new(Palette::ACCENT, Palette::BASE).bold())),
        None,
    );
}

fn draw_form_row(
    screen: &mut Screen,
    x: usize,
    y: usize,
    width: usize,
    label: &str,
    field: &TextField,
    focused: bool,
) {
    let bg = if focused {
        Palette::SURFACE0
    } else {
        Palette::MANTLE
    };
    screen.fill_rect(x, y, width + 9, 1, ' ', Style::new(Palette::TEXT, bg));
    screen.put(
        x,
        y,
        label,
        if focused {
            Style::new(Palette::ACCENT, bg).bold()
        } else {
            Style::new(Palette::MUTED, bg).dim()
        },
    );
    let chars = field.chars();
    let cursor = field.cursor();
    let widths: Vec<usize> = chars
        .iter()
        .map(|ch| util::width(&ch.to_string()))
        .collect();
    let available = width.saturating_sub(1).max(2);
    let mut start = 0;
    while start < cursor {
        let used: usize = widths[start..cursor].iter().sum();
        if used < available {
            break;
        }
        start += 1;
    }
    let mut shown = String::new();
    let mut used = 0;
    for index in start..chars.len() {
        if used + widths[index] > available {
            break;
        }
        shown.push(chars[index]);
        used += widths[index];
    }
    let field_x = x + 9;
    screen.put(field_x, y, &shown, Style::new(Palette::TEXT, bg));
    let caret_offset: usize = widths[start..cursor].iter().sum();
    let caret_x = field_x + caret_offset;
    if focused && caret_x < x + 9 + width {
        screen.put(caret_x, y, "▏", Style::new(Palette::ACCENT, bg).bold());
    }
}

fn centered(rows: usize, cols: usize, width: usize, height: usize) -> (usize, usize) {
    let x = cols.saturating_sub(width) / 2;
    let y = rows.saturating_sub(height) / 2;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Key, KeyCode};

    fn screen_text(app: &App, rows: usize, cols: usize) -> Vec<String> {
        render_lines(app, rows, cols)
    }

    fn test_app() -> App {
        use crate::model::{Counts, Pane, Space, Status, TabEntry};
        use std::time::Duration;
        let mut app = App::new(Some("/home/me".to_owned()));
        let mut counts = Counts::default();
        counts.add(Status::NeedsYou);
        counts.add(Status::Working);
        counts.add(Status::Done);
        app.spaces = vec![
            Space {
                name: "dots".to_owned(),
                is_current: true,
                tabs: vec![
                    TabEntry {
                        position: 0,
                        name: "doftfiles".to_owned(),
                    },
                    TabEntry {
                        position: 1,
                        name: "home".to_owned(),
                    },
                ],
                panes: vec![
                    Pane {
                        id: 1,
                        tab_position: 0,
                        tab_name: "doftfiles".to_owned(),
                        title: "opencode title".to_owned(),
                        app: "opencode".to_owned(),
                        status: Status::Working,
                        base_status: Status::Working,
                        base_text: "editing auth".to_owned(),
                        text: "editing auth".to_owned(),
                        hook: None,
                        cwd: Some("/home/me/dev/dots".to_owned()),
                        age_ms: 12_000,
                        base_age_ms: 12_000,
                        is_focused: true,
                    },
                    Pane {
                        id: 2,
                        tab_position: 1,
                        tab_name: "home".to_owned(),
                        title: "claude title".to_owned(),
                        app: "claude".to_owned(),
                        status: Status::NeedsYou,
                        base_status: Status::NeedsYou,
                        base_text: "approve the migration".to_owned(),
                        text: "approve the migration".to_owned(),
                        hook: None,
                        cwd: Some("/home/me/dev/dots".to_owned()),
                        age_ms: 90_000,
                        base_age_ms: 90_000,
                        is_focused: false,
                    },
                ],
                cwd: Some("/home/me/dev/dots".to_owned()),
                counts,
                status: Status::NeedsYou,
                connected_clients: 1,
                created: Duration::from_secs(120),
            },
            Space {
                name: "server".to_owned(),
                is_current: false,
                tabs: vec![TabEntry {
                    position: 0,
                    name: "logs".to_owned(),
                }],
                panes: vec![Pane {
                    id: 3,
                    tab_position: 0,
                    tab_name: "logs".to_owned(),
                    title: "zsh".to_owned(),
                    app: "shell".to_owned(),
                    status: Status::Idle,
                    base_status: Status::Idle,
                    base_text: String::new(),
                    text: String::new(),
                    hook: None,
                    cwd: Some("/srv/app".to_owned()),
                    age_ms: 0,
                    base_age_ms: 0,
                    is_focused: false,
                }],
                cwd: Some("/srv/app".to_owned()),
                counts: Counts {
                    idle: 1,
                    ..Counts::default()
                },
                status: Status::Idle,
                connected_clients: 1,
                created: Duration::from_secs(300),
            },
        ];
        app.current = Some("dots".to_owned());
        app.exited = vec![crate::model::ExitedSpace {
            name: "old-work".to_owned(),
            age: Duration::from_secs(120),
        }];
        app.rebuild_rows();
        app
    }

    #[test]
    fn render_has_a_header_list_and_footer() {
        let app = test_app();
        let lines = screen_text(&app, 30, 100);
        assert_eq!(lines.len(), 30);
        assert!(lines[0].contains("Spaces"));
        assert!(lines.iter().any(|line| line.contains("opencode")));
        assert!(lines
            .iter()
            .any(|line| line.contains("approve the migration")));
        assert!(lines.iter().any(|line| line.contains("EXITED")));
    }

    #[test]
    fn every_rendered_line_is_exactly_the_panel_width() {
        let app = test_app();
        for (rows, cols) in [(30, 100), (20, 80), (14, 60), (12, 40)] {
            for line in screen_text(&app, rows, cols) {
                assert_eq!(util::width(&line), cols);
            }
        }
    }

    #[test]
    fn search_mode_shows_the_query_and_caret() {
        let mut app = test_app();
        app.configure_search("claude");
        let lines = screen_text(&app, 20, 100);
        assert!(lines[1].contains("claude"));
        app.handle_key(Key {
            code: KeyCode::End,
            ctrl: false,
            alt: false,
            shift: false,
        });
        let lines = screen_text(&app, 20, 100);
        assert!(lines[1].contains("_"));
    }

    #[test]
    fn dialogs_are_drawn_over_the_list() {
        let mut app = test_app();
        app.open_new(Some("fresh"));
        let lines = screen_text(&app, 30, 100);
        assert!(lines.iter().any(|line| line.contains("New space")));
        assert!(lines
            .iter()
            .any(|line| line.contains("The folder is the starting directory")));
    }
}
