//! Plugin registry view — searchable list of available plugins from the remote registry.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};
use rust_i18n::t;
use tuiserial_core::AppState;

/// Draw the registry view inside the plugin modal.
pub(crate) fn draw_registry_view(f: &mut Frame, app: &AppState, modal_area: Rect) {
    let title = format!(" {} ", t!("plugin.registry.title"));
    let inner = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title)
        .title_alignment(Alignment::Left);

    let inner_area = inner.inner(modal_area);
    f.render_widget(inner, modal_area);

    let search_height = 1u16;
    let hint_height = 1u16;
    let table_height = inner_area
        .height
        .saturating_sub(search_height + hint_height + 1);

    let search_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y,
        width: inner_area.width.saturating_sub(2),
        height: 1,
    };

    let table_area = Rect {
        x: inner_area.x,
        y: inner_area.y + search_height,
        width: inner_area.width,
        height: table_height.min(inner_area.height.saturating_sub(search_height)),
    };

    let hint_area = Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height.saturating_sub(hint_height),
        width: inner_area.width,
        height: hint_height,
    };

    draw_search_bar(f, app, search_area);

    if app.registry_loading {
        draw_registry_loading(f, table_area);
    } else if app.registry_entries.is_empty() {
        draw_registry_empty(f, table_area);
    } else {
        draw_registry_table(f, app, table_area);
    }

    draw_registry_hints(f, hint_area);
}

fn draw_search_bar(f: &mut Frame, app: &AppState, area: Rect) {
    let placeholder = t!("plugin.registry.search_placeholder");
    let display = if app.registry_search_query.is_empty() {
        Span::styled(placeholder, Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            &app.registry_search_query,
            Style::default().fg(Color::Yellow),
        )
    };

    let cursor = Span::styled("▌", Style::default().fg(Color::Cyan));

    let line = Line::from(vec![
        Span::styled("🔍 ", Style::default().fg(Color::Cyan)),
        display,
        cursor,
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

fn draw_registry_loading(f: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            t!("plugin.registry.loading"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let para = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn draw_registry_empty(f: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            t!("plugin.registry.empty"),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let para = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn draw_registry_table(f: &mut Frame, app: &AppState, area: Rect) {
    let query = app.registry_search_query.to_lowercase();
    let filtered: Vec<&tuiserial_core::RegistryEntry> = if query.is_empty() {
        app.registry_entries.iter().collect()
    } else {
        app.registry_entries
            .iter()
            .filter(|e| {
                e.name.to_lowercase().contains(&query)
                    || e.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect()
    };

    if filtered.is_empty() {
        draw_registry_empty(f, area);
        return;
    }

    let header_height: u16 = 3;
    let visible_rows = area.height.saturating_sub(header_height) as usize;
    let total = filtered.len();

    let scroll = app
        .registry_scroll
        .min(total.saturating_sub(visible_rows.max(1)));

    let mut table_state = TableState::default();
    if total > 0 {
        table_state.select(Some(scroll));
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let header = Row::new(vec![
        Span::styled(t!("plugin.modal.name"), header_style),
        Span::styled("Description", header_style),
        Span::styled("Author", header_style),
    ]);

    let usable = area.width.saturating_sub(6);
    let name_w = (usable * 25 / 100).max(14);
    let author_w = (usable * 25 / 100).max(10);
    let desc_w = usable.saturating_sub(name_w + author_w);

    let widths = [
        ratatui::layout::Constraint::Length(name_w),
        ratatui::layout::Constraint::Length(desc_w),
        ratatui::layout::Constraint::Length(author_w),
    ];

    let rows: Vec<Row> = filtered
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .map(|e| registry_row(e, app, name_w as usize, desc_w as usize, author_w as usize))
        .collect();

    let pad_count = visible_rows.saturating_sub(rows.len());
    let mut all_rows = rows;
    for _ in 0..pad_count {
        all_rows.push(Row::new(vec!["", "", ""]));
    }

    let table = Table::new(all_rows, widths)
        .header(header)
        .block(Block::default())
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .column_spacing(1);

    f.render_stateful_widget(table, area, &mut table_state);
}

fn registry_row<'a>(
    entry: &'a tuiserial_core::RegistryEntry,
    app: &AppState,
    name_w: usize,
    desc_w: usize,
    author_w: usize,
) -> Row<'a> {
    let installed = app.plugin_statuses.iter().any(|s| s.name == entry.name);

    let name_display = if installed {
        format!("{} ✓", entry.name)
    } else {
        entry.name.clone()
    };
    let name_style = if installed {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::White)
    };

    let desc = entry.description.as_deref().unwrap_or("—");
    let desc_display = if desc.len() > desc_w {
        format!("{}…", &desc[..desc_w.saturating_sub(1)])
    } else {
        desc.to_string()
    };

    let author = entry.author.as_deref().unwrap_or("—");
    let author_display = if author.len() > author_w {
        format!("{}…", &author[..author_w.saturating_sub(1)])
    } else {
        author.to_string()
    };

    Row::new(vec![
        Span::styled(
            if name_display.len() > name_w {
                format!("{}…", &name_display[..name_w.saturating_sub(1)])
            } else {
                name_display
            },
            name_style,
        ),
        Span::styled(desc_display, Style::default().fg(Color::DarkGray)),
        Span::styled(author_display, Style::default().fg(Color::DarkGray)),
    ])
}

fn draw_registry_hints(f: &mut Frame, area: Rect) {
    let hints = vec![
        Span::styled(
            t!("plugin.registry.hint_search"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            t!("plugin.registry.hint_install"),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  "),
        Span::styled(
            t!("plugin.registry.hint_back"),
            Style::default().fg(Color::Red),
        ),
    ];

    let para = Paragraph::new(Line::from(hints)).alignment(Alignment::Center);
    f.render_widget(para, area);
}
