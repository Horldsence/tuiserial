//! Configuration panel rendering - dropdowns for serial port settings
//!
//! This module handles rendering of all configuration dropdowns including
//! port selection, baud rate, data bits, parity, stop bits, and flow control.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tuiserial_core::{i18n::t, AppState, FlowControl, FocusedField, Language, MenuState, Parity};

use crate::areas::{update_area, UiAreaField};

/// Draw the serial port selection dropdown
pub fn draw_port_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::Port, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::Port && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(" {} [{}] ", t("label.port", lang), t("label.locked", lang))
    } else if focused {
        format!(
            " {} [↑↓ {} | r {}] ",
            t("label.port", lang),
            t("hint.select", lang),
            t("hint.refresh", lang)
        )
    } else {
        format!(" {} ", t("label.port", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    if app.ports.is_empty() {
        let empty_text = format!(
            "{}\n{} r {}",
            if lang == Language::English {
                "No ports available"
            } else {
                "无可用串口"
            },
            if lang == Language::English {
                "Press"
            } else {
                "按"
            },
            t("hint.refresh", lang)
        );
        let para = Paragraph::new(empty_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title.as_str())
                    .border_style(style),
            )
            .style(Style::default().fg(Color::Red));
        f.render_widget(para, area);
        return;
    }

    let items: Vec<ListItem> = app
        .ports
        .iter()
        .map(|p| ListItem::new(p.as_str()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.port_list_state.clone());
}

/// Draw the baud rate selection dropdown
pub fn draw_baud_rate_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::BaudRate, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::BaudRate && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(
            " {} [{}] ",
            t("label.baud_rate", lang),
            t("label.locked", lang)
        )
    } else if focused {
        format!(
            " {} [←→ {}] ",
            t("label.baud_rate", lang),
            t("hint.switch", lang)
        )
    } else {
        format!(" {} ", t("label.baud_rate", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .baud_rate_options
        .iter()
        .map(|b| ListItem::new(b.to_string()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.baud_rate_state.clone());
}

/// Draw the data bits selection dropdown
pub fn draw_data_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::DataBits, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::DataBits && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(
            " {} [{}] ",
            t("label.data_bits", lang),
            t("label.locked", lang)
        )
    } else if focused {
        format!(
            " {} [↑↓ {}] ",
            t("label.data_bits", lang),
            t("hint.select", lang)
        )
    } else {
        format!(" {} ", t("label.data_bits", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .data_bits_options
        .iter()
        .map(|b| ListItem::new(format!("{} bits", b)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, area, &mut app.data_bits_state.clone());
}

/// Draw the parity selection dropdown
pub fn draw_parity_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::Parity, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::Parity && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(
            " {} [{}] ",
            t("label.parity", lang),
            t("label.locked", lang)
        )
    } else if focused {
        format!(
            " {} [↑↓ {} | p {}] ",
            t("label.parity", lang),
            t("hint.select", lang),
            t("hint.toggle", lang)
        )
    } else {
        format!(" {} ", t("label.parity", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .parity_options
        .iter()
        .map(|p| {
            let text = match p {
                Parity::None => t("parity.none", lang),
                Parity::Even => t("parity.even", lang),
                Parity::Odd => t("parity.odd", lang),
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.parity_state.clone());
}

/// Draw the stop bits selection dropdown
pub fn draw_stop_bits_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::StopBits, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::StopBits && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(
            " {} [{}] ",
            t("label.stop_bits", lang),
            t("label.locked", lang)
        )
    } else if focused {
        format!(
            " {} [↑↓ {}] ",
            t("label.stop_bits", lang),
            t("hint.select", lang)
        )
    } else {
        format!(" {} ", t("label.stop_bits", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .stop_bits_options
        .iter()
        .map(|s| ListItem::new(format!("{} bit", s)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title.as_str())
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.stop_bits_state.clone());
}

/// Draw the flow control selection dropdown
pub fn draw_flow_control_dropdown(f: &mut Frame, app: &AppState, area: Rect) {
    // Store area for mouse interaction
    update_area(UiAreaField::FlowControl, area);

    // Don't expand dropdown if menu is open
    let menu_open = !matches!(app.menu_state, MenuState::None);
    let focused = app.focused_field == FocusedField::FlowControl && !menu_open;
    let is_locked = app.config_locked;

    let lang = app.language;
    let title = if is_locked {
        format!(
            " {} [{}] ",
            t("label.flow_control", lang),
            t("label.locked", lang)
        )
    } else if focused {
        format!(
            " {} [↑↓ {} | f {}] ",
            t("label.flow_control", lang),
            t("hint.select", lang),
            t("hint.toggle", lang)
        )
    } else {
        format!(" {} ", t("label.flow_control", lang))
    };

    let style = if is_locked {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let items: Vec<ListItem> = app
        .flow_control_options
        .iter()
        .map(|fc| {
            let text = match fc {
                FlowControl::None => "None",
                FlowControl::Hardware => "Hardware",
                FlowControl::Software => "Software",
            };
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.flow_control_state.clone());
}
