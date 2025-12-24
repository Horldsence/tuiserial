//! Keyboard shortcuts help overlay
//!
//! This module provides a help overlay that displays all available keyboard shortcuts
//! in a centered popup window.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tuiserial_core::{i18n::t, Language};

/// Draw the help overlay centered on the screen
pub fn draw_help_overlay(f: &mut Frame, lang: Language) {
    let area = f.area();

    // Calculate centered overlay area (80% width, 90% height)
    let overlay_width = (area.width * 80) / 100;
    let overlay_height = (area.height * 90) / 100;
    let x = (area.width.saturating_sub(overlay_width)) / 2;
    let y = (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect {
        x,
        y,
        width: overlay_width,
        height: overlay_height,
    };

    // Clear the area
    f.render_widget(Clear, overlay_area);

    // Create help content
    let help_lines = create_help_content(lang);

    // Create the help widget
    let help_widget = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .title(Span::styled(
                    format!(" {} ", t("shortcuts.title", lang)),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    f.render_widget(help_widget, overlay_area);

    // Draw footer hint
    let footer_area = Rect {
        x: overlay_area.x,
        y: overlay_area.y + overlay_area.height - 1,
        width: overlay_area.width,
        height: 1,
    };

    let footer_text = if lang == Language::English {
        " Press ESC or F1 to close "
    } else {
        " 按 ESC 或 F1 关闭 "
    };

    let footer = Paragraph::new(footer_text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);

    // Create a small rect for the footer inside the border
    let footer_inner = Rect {
        x: footer_area.x + 2,
        y: footer_area.y,
        width: footer_area.width.saturating_sub(4),
        height: 1,
    };

    f.render_widget(footer, footer_inner);
}

/// Create help content with all keyboard shortcuts
fn create_help_content(lang: Language) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Title spacer
    lines.push(Line::from(""));

    // Session Management
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", t("shortcuts.session", lang)),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    add_shortcut_line(&mut lines, "Ctrl+T", t("shortcuts.new_session", lang));
    add_shortcut_line(&mut lines, "Ctrl+W", t("shortcuts.close_session", lang));
    add_shortcut_line(
        &mut lines,
        "Ctrl+Tab / Ctrl+→",
        t("shortcuts.next_session", lang),
    );
    add_shortcut_line(
        &mut lines,
        "Ctrl+Shift+Tab / Ctrl+←",
        t("shortcuts.prev_session", lang),
    );
    add_shortcut_line(&mut lines, "Ctrl+1~9", t("shortcuts.switch_1_9", lang));
    add_shortcut_line(
        &mut lines,
        "F2",
        if lang == Language::English {
            "Rename Session"
        } else {
            "重命名会话"
        },
    );

    lines.push(Line::from(""));

    // Layout Management
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", t("shortcuts.layout", lang)),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    add_shortcut_line(&mut lines, "Ctrl+L", t("shortcuts.cycle_layout", lang));
    add_shortcut_line(&mut lines, "Ctrl+Shift+L", t("shortcuts.prev_layout", lang));
    add_shortcut_line(&mut lines, "Ctrl+P", t("shortcuts.next_pane", lang));
    add_shortcut_line(
        &mut lines,
        "Ctrl+Shift+P",
        t("shortcuts.prev_pane_key", lang),
    );
    add_shortcut_line(
        &mut lines,
        "Ctrl+N",
        t("shortcuts.cycle_pane_session", lang),
    );

    lines.push(Line::from(""));

    // General Operations
    lines.push(Line::from(vec![Span::styled(
        format!("  {}", t("shortcuts.general", lang)),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    add_shortcut_line(&mut lines, "O", t("shortcuts.connect", lang));
    add_shortcut_line(&mut lines, "C", t("shortcuts.clear", lang));
    add_shortcut_line(&mut lines, "X", t("shortcuts.display_mode", lang));
    add_shortcut_line(&mut lines, "A", t("shortcuts.auto_scroll", lang));
    add_shortcut_line(&mut lines, "F10", t("shortcuts.menu", lang));
    add_shortcut_line(
        &mut lines,
        "F1",
        if lang == Language::English {
            "Toggle Help"
        } else {
            "切换帮助"
        },
    );
    add_shortcut_line(&mut lines, "Ctrl+Q", t("shortcuts.quit", lang));

    lines.push(Line::from(""));

    // Navigation
    lines.push(Line::from(vec![Span::styled(
        if lang == Language::English {
            "  Navigation:"
        } else {
            "  导航："
        },
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    add_shortcut_line(&mut lines, "Tab", t("help.tab", lang));
    add_shortcut_line(&mut lines, "Shift+Tab", t("help.shift_tab", lang));
    add_shortcut_line(
        &mut lines,
        "↑/↓",
        if lang == Language::English {
            "Navigate items"
        } else {
            "导航选项"
        },
    );
    add_shortcut_line(&mut lines, "Enter", t("help.enter", lang));
    add_shortcut_line(&mut lines, "Esc", t("help.esc", lang));

    lines.push(Line::from(""));

    // Configuration Menu
    lines.push(Line::from(vec![Span::styled(
        if lang == Language::English {
            "  Configuration:"
        } else {
            "  配置："
        },
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    add_shortcut_line(
        &mut lines,
        "Ctrl+S",
        if lang == Language::English {
            "Save Config"
        } else {
            "保存配置"
        },
    );
    add_shortcut_line(
        &mut lines,
        "Ctrl+O",
        if lang == Language::English {
            "Load Config"
        } else {
            "加载配置"
        },
    );
    add_shortcut_line(
        &mut lines,
        "Ctrl+Shift+L",
        if lang == Language::English {
            "Toggle Language"
        } else {
            "切换语言"
        },
    );

    lines.push(Line::from(""));

    // Mouse Support
    lines.push(Line::from(vec![Span::styled(
        if lang == Language::English {
            "  Mouse Support:"
        } else {
            "  鼠标支持："
        },
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    let mouse_hints = if lang == Language::English {
        vec![
            "    • Click on tabs to switch sessions",
            "    • Click on dropdowns to select options",
            "    • Click on buttons to execute actions",
            "    • Scroll wheel to navigate log area",
        ]
    } else {
        vec![
            "    • 点击标签页切换会话",
            "    • 点击下拉框选择选项",
            "    • 点击按钮执行操作",
            "    • 滚轮滚动日志区域",
        ]
    };

    for hint in mouse_hints {
        lines.push(Line::from(vec![Span::styled(
            hint,
            Style::default().fg(Color::DarkGray),
        )]));
    }

    lines.push(Line::from(""));

    // Footer tip
    lines.push(Line::from(""));
    let tip = if lang == Language::English {
        "  💡 Tip: Most operations can be performed via mouse clicks or keyboard shortcuts"
    } else {
        "  💡 提示：大部分操作可以通过鼠标点击或键盘快捷键完成"
    };
    lines.push(Line::from(vec![Span::styled(
        tip,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::ITALIC),
    )]));

    lines
}

/// Add a shortcut line with key and description
fn add_shortcut_line(lines: &mut Vec<Line<'static>>, key: &str, description: &str) {
    // Extract description text if it contains ":" from i18n
    let desc = if description.contains(':') {
        description
            .split(':')
            .last()
            .unwrap_or(description)
            .trim()
            .to_string()
    } else {
        description.to_string()
    };

    lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(
            format!("{:20}", key),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(Color::White)),
    ]));
}

/// Draw a compact help hint in the status bar
pub fn draw_help_hint(f: &mut Frame, area: Rect, lang: Language) {
    let hint_text = if lang == Language::English {
        " F1: Help | F10: Menu | Ctrl+Q: Quit "
    } else {
        " F1: 帮助 | F10: 菜单 | Ctrl+Q: 退出 "
    };

    let hint = Paragraph::new(hint_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);

    f.render_widget(hint, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_content_generation() {
        let lines_en = create_help_content(Language::English);
        let lines_zh = create_help_content(Language::Chinese);

        assert!(!lines_en.is_empty());
        assert!(!lines_zh.is_empty());
        assert_eq!(lines_en.len(), lines_zh.len());
    }
}
