//! Input utility functions — text width, hex input rebuilding, and paste handling.

use tuiserial_core::{AppState, FocusedField, TxMode};

/// Calculate display width of a string (handles CJK characters).
pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// Rebuild hex-mode input with auto-spacing: extract hex digits, group in pairs with spaces.
/// Preserves cursor position relative to hex content.
pub fn rebuild_hex_input(app: &mut AppState) {
    let hex_only: String = app
        .tx_input
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();

    let hex_before_cursor: usize = app.tx_input[..app
        .tx_input
        .char_indices()
        .nth(app.tx_cursor)
        .map(|(i, _)| i)
        .unwrap_or(app.tx_input.len())]
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .count();

    let mut new_input = String::new();
    for (i, ch) in hex_only.chars().enumerate() {
        new_input.push(ch);
        if i % 2 == 1 && i < hex_only.len() - 1 {
            new_input.push(' ');
        }
    }

    let mut hex_count = 0;
    let mut new_cursor = 0;
    for (i, ch) in new_input.chars().enumerate() {
        if ch.is_ascii_hexdigit() {
            hex_count += 1;
        }
        if hex_count == hex_before_cursor {
            new_cursor = i + 1;
            break;
        }
    }

    app.tx_input = new_input;
    app.tx_cursor = new_cursor;
}

/// Handle paste events: in hex mode filter non-hex chars and rebuild spacing; in ASCII insert as-is.
pub fn handle_paste_event(data: &str, app: &mut AppState) {
    if app.focused_field != FocusedField::TxInput {
        return;
    }

    if app.tx_mode == TxMode::Hex {
        let hex_only: String = data
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();
        if !hex_only.is_empty() {
            let byte_idx = app
                .tx_input
                .char_indices()
                .nth(app.tx_cursor)
                .map(|(i, _)| i)
                .unwrap_or(app.tx_input.len());
            app.tx_input.insert_str(byte_idx, &hex_only);
            app.tx_cursor += hex_only.chars().count();
            rebuild_hex_input(app);
        }
    } else {
        let byte_idx = app
            .tx_input
            .char_indices()
            .nth(app.tx_cursor)
            .map(|(i, _)| i)
            .unwrap_or(app.tx_input.len());
        app.tx_input.insert_str(byte_idx, data);
        app.tx_cursor += data.chars().count();
    }
}
