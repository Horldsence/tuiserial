//! Plugin manager modal — displays plugin load status, errors, and hooks.
//!
//! Supports two modes:
//! - **Local** — installed plugins with status, hooks, and errors.
//! - **Registry** — searchable list of available plugins from the remote registry.
//!
//! Registry view lives in the sibling [`super::plugin_registry`] module.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Frame,
};
use rust_i18n::t;
use tuiserial_core::{AppState, PluginLoadState, PluginModalMode};

use crate::areas::{update_area, UiAreaField};

/// Draw the plugin manager modal overlay.
///
/// Centers a modal in the middle of the terminal. In Local mode it shows
/// installed plugins with status/hooks/errors. In Registry mode it shows
/// a searchable list of available plugins from the registry.
pub fn draw_plugin_modal(f: &mut Frame, app: &AppState) {
    let area = f.area();

    // Modal dimensions: 80% of terminal width, up to 90% height
    let modal_width = (area.width * 4 / 5).clamp(50, 90);
    let modal_height = (area.height * 9 / 10).clamp(12, 30);
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };

    // Store area for mouse interaction
    update_area(UiAreaField::PluginModal, modal_area);

    // Clear the area first
    f.render_widget(Clear, modal_area);

    match app.plugin_modal_mode {
        PluginModalMode::Registry => crate::plugin_registry::draw_registry_view(f, app, modal_area),
        PluginModalMode::Local => draw_local_view(f, app, modal_area),
    }
}

// ── Local (installed plugins) view ───────────────────────────────

fn draw_local_view(f: &mut Frame, app: &AppState, modal_area: Rect) {
    let title = format!(" {} ", t!("plugin.modal.title"));
    let inner = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title)
        .title_alignment(Alignment::Left);

    let inner_area = inner.inner(modal_area);

    // Split inner area: table takes most space, hints at bottom
    let table_height = inner_area.height.saturating_sub(2); // 2 lines for hint bar
    let table_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: table_height.min(inner_area.height),
    };

    let hint_area = Rect {
        x: inner_area.x,
        y: inner_area.y + table_height,
        width: inner_area.width,
        height: 1,
    };

    f.render_widget(inner, modal_area);

    if app.plugin_statuses.is_empty() {
        draw_local_empty(f, inner_area);
    } else {
        draw_local_table(f, app, table_area);
    }

    draw_local_hints(f, hint_area);
}

fn draw_local_empty(f: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(""),
        Line::from(Span::styled(
            t!("plugin.modal.no_plugins"),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Plugins go in ~/.config/tuiserial/plugins/<name>/plugin.ts",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let para = Paragraph::new(msg).alignment(Alignment::Center);
    f.render_widget(para, area);
}

fn draw_local_table(f: &mut Frame, app: &AppState, area: Rect) {
    let header_height: u16 = 3;
    let visible_rows = area.height.saturating_sub(header_height) as usize;
    let total = app.plugin_statuses.len();

    let scroll = app.plugin_modal_scroll.min(total.saturating_sub(visible_rows.max(1)));

    let mut table_state = TableState::default();
    if total > 0 {
        table_state.select(Some(scroll));
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let header = Row::new(vec![
        Span::styled(t!("plugin.modal.status"), header_style),
        Span::styled(t!("plugin.modal.name"), header_style),
        Span::styled(t!("plugin.modal.hooks"), header_style),
        Span::styled(t!("plugin.modal.error"), header_style),
    ]);

    let name_width = (area.width.saturating_sub(8 + 16 + 13)) as usize;
    let name_width = name_width.max(12);

    let status_w = 8u16;
    let hooks_w = 16u16;
    let error_w = area.width.saturating_sub(status_w + name_width as u16 + hooks_w + 6);

    let widths = [
        ratatui::layout::Constraint::Length(status_w),
        ratatui::layout::Constraint::Length(name_width as u16),
        ratatui::layout::Constraint::Length(hooks_w),
        ratatui::layout::Constraint::Length(error_w),
    ];

    let rows: Vec<Row> = app
        .plugin_statuses
        .iter()
        .skip(scroll)
        .take(visible_rows)
        .map(|ps| local_row(ps, name_width))
        .collect();

    let pad_count = visible_rows.saturating_sub(rows.len());
    let mut all_rows = rows;
    for _ in 0..pad_count {
        all_rows.push(Row::new(vec!["", "", "", ""]));
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

fn local_row<'a>(ps: &'a tuiserial_core::PluginLoadStatus, name_width: usize) -> Row<'a> {
    let (status_text, status_color) = match ps.state {
        PluginLoadState::Loading => ("⏳", Color::Yellow),
        PluginLoadState::Loaded => ("✓", Color::Green),
        PluginLoadState::Error => ("✗", Color::Red),
        PluginLoadState::Disabled => ("—", Color::DarkGray),
    };

    let name_display = if ps.name.len() > name_width {
        format!("{}…", &ps.name[..name_width.saturating_sub(1)])
    } else {
        ps.name.clone()
    };

    let mut hook_parts: Vec<String> = Vec::new();
    if ps.has_rx_hook {
        hook_parts.push("RX".to_string());
    }
    if ps.has_tx_hook {
        hook_parts.push("TX".to_string());
    }
    if ps.has_connect_hook {
        hook_parts.push("CN".to_string());
    }
    if ps.has_disconnect_hook {
        hook_parts.push("DC".to_string());
    }
    let hooks_display = if hook_parts.is_empty() {
        "—".to_string()
    } else {
        hook_parts.join(" ")
    };

    let error_display = match (&ps.error_message, &ps.metadata) {
        (Some(err), _) => {
            let err = err.replace('\n', " ");
            if err.len() > 40 {
                format!("{}…", &err[..39])
            } else {
                err
            }
        }
        (None, Some(meta)) => {
            if let Some(desc) = &meta.description {
                if desc.len() > 40 {
                    format!("{}…", &desc[..39])
                } else {
                    desc.clone()
                }
            } else {
                String::new()
            }
        }
        (None, None) => String::new(),
    };

    let style = Style::default().fg(match ps.state {
        PluginLoadState::Loaded => Color::White,
        PluginLoadState::Error => Color::Red,
        PluginLoadState::Disabled => Color::DarkGray,
        PluginLoadState::Loading => Color::Yellow,
    });

    Row::new(vec![
        Span::styled(status_text, Style::default().fg(status_color)),
        Span::styled(name_display, style),
        Span::styled(hooks_display, style),
        Span::styled(
            error_display,
            Style::default().fg(if ps.state == PluginLoadState::Error {
                Color::Red
            } else {
                Color::DarkGray
            }),
        ),
    ])
}

fn draw_local_hints(f: &mut Frame, area: Rect) {
    let hints = vec![
        Span::styled(
            t!("plugin.modal.hint_navigate"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled(
            t!("plugin.modal.hint_reload"),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            t!("plugin.modal.hint_close"),
            Style::default().fg(Color::Red),
        ),
    ];

    let para = Paragraph::new(Line::from(hints)).alignment(Alignment::Center);
    f.render_widget(para, area);
}
