//! Menu definition and structure
//!
//! This module provides a centralized, type-safe menu structure that eliminates
//! hardcoded indices and simplifies menu navigation and interaction.
//!
//! Following Linus's principle: "Bad programmers worry about the code.
//! Good programmers worry about data structures."

use crate::Language;

/// Calculate display width of a string (handles CJK characters)
///
/// CJK (Chinese, Japanese, Korean) characters take up 2 display columns,
/// while ASCII characters take up 1 column.
fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// Menu action that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    // File menu
    SaveConfig,
    LoadConfig,
    Exit,

    // Session menu (for multi-session support)
    NewSession,
    DuplicateSession,
    RenameSession,
    CloseSession,

    // View menu (for layout support)
    ViewSingle,
    ViewSplitHorizontal,
    ViewSplitVertical,
    ViewGrid2x2,
    ViewNextPane,
    ViewPrevPane,

    // Settings menu
    ToggleLanguage,

    // Help menu
    ShowShortcuts,
    ShowAbout,

    // Special
    Separator, // Not an action, just for display
}

impl MenuAction {
    /// Get the translation key for this action
    pub fn label_key(&self) -> &'static str {
        match self {
            MenuAction::SaveConfig => "menu.file.save_config",
            MenuAction::LoadConfig => "menu.file.load_config",
            MenuAction::Exit => "menu.file.exit",
            MenuAction::NewSession => "menu.session.new",
            MenuAction::DuplicateSession => "menu.session.duplicate",
            MenuAction::RenameSession => "menu.session.rename",
            MenuAction::CloseSession => "menu.session.close",
            MenuAction::ViewSingle => "menu.view.single",
            MenuAction::ViewSplitHorizontal => "menu.view.split_h",
            MenuAction::ViewSplitVertical => "menu.view.split_v",
            MenuAction::ViewGrid2x2 => "menu.view.grid_2x2",
            MenuAction::ViewNextPane => "menu.view.next_pane",
            MenuAction::ViewPrevPane => "menu.view.prev_pane",
            MenuAction::ToggleLanguage => "menu.settings.toggle_language",
            MenuAction::ShowShortcuts => "menu.help.shortcuts",
            MenuAction::ShowAbout => "menu.help.about",
            MenuAction::Separator => "",
        }
    }

    /// Check if this is a separator
    pub fn is_separator(&self) -> bool {
        matches!(self, MenuAction::Separator)
    }
}

/// A single menu with its items
#[derive(Debug, Clone)]
pub struct Menu {
    pub label_key: &'static str,
    pub items: &'static [MenuAction],
}

impl Menu {
    /// Get the number of items (including separators)
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get the item at index
    pub fn get_item(&self, index: usize) -> Option<MenuAction> {
        self.items.get(index).copied()
    }
}

/// All application menus
pub struct MenuBar {
    pub menus: &'static [Menu],
}

impl MenuBar {
    /// Get the number of menus
    pub fn menu_count(&self) -> usize {
        self.menus.len()
    }

    /// Get a menu by index
    pub fn get_menu(&self, index: usize) -> Option<&Menu> {
        self.menus.get(index)
    }

    /// Get menu label key
    pub fn get_menu_label_key(&self, index: usize) -> Option<&'static str> {
        self.menus.get(index).map(|m| m.label_key)
    }

    /// Get item count for a menu
    pub fn get_item_count(&self, menu_index: usize) -> usize {
        self.menus
            .get(menu_index)
            .map(|m| m.item_count())
            .unwrap_or(0)
    }

    /// Get a specific menu action
    pub fn get_action(&self, menu_index: usize, item_index: usize) -> Option<MenuAction> {
        self.menus
            .get(menu_index)
            .and_then(|m| m.get_item(item_index))
    }
}

// Menu definitions - the single source of truth
const FILE_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::SaveConfig,
    MenuAction::LoadConfig,
    MenuAction::Separator,
    MenuAction::Exit,
];

const SESSION_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::NewSession,
    MenuAction::DuplicateSession,
    MenuAction::RenameSession,
    MenuAction::Separator,
    MenuAction::CloseSession,
];

const VIEW_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::ViewSingle,
    MenuAction::ViewSplitHorizontal,
    MenuAction::ViewSplitVertical,
    MenuAction::ViewGrid2x2,
    MenuAction::Separator,
    MenuAction::ViewNextPane,
    MenuAction::ViewPrevPane,
];

const SETTINGS_MENU_ITEMS: &[MenuAction] = &[MenuAction::ToggleLanguage];

const HELP_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::ShowShortcuts,
    MenuAction::Separator,
    MenuAction::ShowAbout,
];

// All menus in order
const ALL_MENUS: &[Menu] = &[
    Menu {
        label_key: "menu.file",
        items: FILE_MENU_ITEMS,
    },
    Menu {
        label_key: "menu.session",
        items: SESSION_MENU_ITEMS,
    },
    Menu {
        label_key: "menu.view",
        items: VIEW_MENU_ITEMS,
    },
    Menu {
        label_key: "menu.settings",
        items: SETTINGS_MENU_ITEMS,
    },
    Menu {
        label_key: "menu.help",
        items: HELP_MENU_ITEMS,
    },
];

/// Global menu bar instance
pub const MENU_BAR: MenuBar = MenuBar { menus: ALL_MENUS };

/// Calculate x offset for a menu in the menu bar
pub fn calculate_menu_x_offset(menu_index: usize, lang: Language) -> u16 {
    use crate::i18n::t;

    let mut offset = 0u16;
    for i in 0..menu_index.min(MENU_BAR.menu_count()) {
        if let Some(label_key) = MENU_BAR.get_menu_label_key(i) {
            let label = t(label_key, lang);
            // Each menu has format " Label " (label + 2 spaces for padding + 2 spaces between menus)
            offset += display_width(label) as u16 + 4;
        }
    }
    offset
}

/// Find which menu was clicked based on x position
pub fn find_clicked_menu(x: u16, lang: Language) -> Option<usize> {
    use crate::i18n::t;

    let mut current_x = 0u16;
    for (i, menu) in MENU_BAR.menus.iter().enumerate() {
        let label = t(menu.label_key, lang);
        let menu_width = display_width(label) as u16 + 2; // " Label "

        if x >= current_x && x < current_x + menu_width {
            return Some(i);
        }

        current_x += menu_width + 2; // +2 for spacing between menus
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_structure() {
        assert_eq!(MENU_BAR.menu_count(), 5);
        assert_eq!(MENU_BAR.get_item_count(0), 4); // File: Save, Load, Sep, Exit
        assert_eq!(MENU_BAR.get_item_count(1), 5); // Session
        assert_eq!(MENU_BAR.get_item_count(2), 7); // View
        assert_eq!(MENU_BAR.get_item_count(3), 1); // Settings
        assert_eq!(MENU_BAR.get_item_count(4), 3); // Help: Shortcuts, Sep, About
    }

    #[test]
    fn test_menu_actions() {
        assert_eq!(MENU_BAR.get_action(0, 0), Some(MenuAction::SaveConfig));
        assert_eq!(MENU_BAR.get_action(0, 3), Some(MenuAction::Exit));
        assert_eq!(MENU_BAR.get_action(4, 0), Some(MenuAction::ShowShortcuts));
    }

    #[test]
    fn test_separator_detection() {
        assert!(MenuAction::Separator.is_separator());
        assert!(!MenuAction::SaveConfig.is_separator());
    }

    #[test]
    fn test_invalid_indices() {
        assert_eq!(MENU_BAR.get_action(99, 0), None);
        assert_eq!(MENU_BAR.get_action(0, 99), None);
    }

    #[test]
    fn test_menu_label_keys() {
        assert_eq!(MENU_BAR.get_menu_label_key(0), Some("menu.file"));
        assert_eq!(MENU_BAR.get_menu_label_key(1), Some("menu.session"));
        assert_eq!(MENU_BAR.get_menu_label_key(4), Some("menu.help"));
    }
}
