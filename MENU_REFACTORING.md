# Menu System Refactoring Summary

## Problem Statement

The original menu system had several critical issues:

1. **Magic Numbers Everywhere**: Menu count hardcoded as `3` in multiple places, but actually needed to be `5`
2. **No Single Source of Truth**: Menu structure scattered across rendering, event handling, and mouse interaction code
3. **Fragile Mouse Interaction**: Click detection relied on manual width calculations that were error-prone
4. **Index Mismatch**: Different parts of code had different assumptions about menu indices

**"Bad programmers worry about the code. Good programmers worry about data structures."** - Linus Torvalds

## Solution: Centralized Menu Definition

### New Data Structure (`tuiserial-core/src/menu_def.rs`)

```rust
pub enum MenuAction {
    SaveConfig,
    LoadConfig,
    Exit,
    NewSession,
    // ... all actions enumerated
}

pub struct Menu {
    pub label_key: &'static str,
    pub items: &'static [MenuAction],
}

pub struct MenuBar {
    pub menus: &'static [Menu],
}

pub const MENU_BAR: MenuBar = MenuBar { menus: ALL_MENUS };
```

**Key Benefits:**
- ✅ Compile-time constant - zero runtime overhead
- ✅ Type-safe action dispatch
- ✅ Single source of truth
- ✅ All menu structure queries go through one API

### Eliminated Complexity

**Before (Bad):**
```rust
// menu.rs
let menus = vec!["File", "Settings", "Help"]; // Wrong count!

// main.rs event handling
match (menu_idx, item_idx) {
    (0, 0) => { /* Save */ }
    (0, 1) => { /* Load */ }
    // ... 40 lines of hardcoded indices
}

// main.rs mouse handling
let menus = vec!["File", "Settings", "Help"]; // Duplicated!
let mut x_offset = 0u16;
for (i, menu) in menus.iter().enumerate() {
    let menu_width = display_width(menu) as u16 + 2;
    if col >= x_offset && col < x_offset + menu_width {
        // ... manual calculation
    }
    x_offset += menu_width + 2;
}

// Another place
let item_count = match menu_idx {
    0 => 4,  // Hardcoded again!
    1 => 1,
    2 => 1,
    _ => 0,
};
```

**After (Good):**
```rust
// All menu queries
MENU_BAR.menu_count()                      // 5
MENU_BAR.get_item_count(0)                 // 4
MENU_BAR.get_action(menu_idx, item_idx)    // Type-safe action

// Mouse click detection
find_clicked_menu(x, lang)  // Centralized calculation

// Action dispatch
match action {
    MenuAction::SaveConfig => { /* handle */ }
    MenuAction::Exit => { /* handle */ }
    // ... exhaustive match, compiler enforced
}
```

## Changes Made

### 1. Core Layer (`tuiserial-core`)

**New File: `src/menu_def.rs`**
- Defines all menus and actions as compile-time constants
- Provides query API (`menu_count()`, `get_action()`, etc.)
- Handles mouse position calculations centrally

**Updated: `src/lib.rs`**
- Exports `MenuAction`, `MenuBar`, `MENU_BAR`

### 2. UI Layer (`tuiserial-ui`)

**Updated: `src/menu.rs`**
- Removed hardcoded menu arrays
- All rendering reads from `MENU_BAR`
- Added `find_clicked_menu()` for mouse interaction
- Dropdown position calculation uses centralized function

**Updated: `src/lib.rs`**
- Exports `find_clicked_menu` for mouse handling

### 3. Application Layer (`tuiserial-cli`)

**Updated: `src/main.rs`**

**`handle_menu_action()` function:**
```rust
// Before: Match on (menu_idx, item_idx) tuples
match (menu_idx, item_idx) {
    (0, 0) => { /* Save */ }
    (4, 2) => { /* About */ }
    // ... magic numbers everywhere
}

// After: Match on action enum
let action = MENU_BAR.get_action(menu_idx, item_idx)?;
match action {
    MenuAction::SaveConfig => { /* Save */ }
    MenuAction::ShowAbout => { /* About */ }
    // ... type-safe, exhaustive
}
```

**`handle_key_event()` function:**
```rust
// Before: Hardcoded menu count
MenuState::MenuBar((selected + 1) % 3)  // WRONG!

// After: Query from definition
MenuState::MenuBar((selected + 1) % MENU_BAR.menu_count())
```

**`handle_mouse_event()` function:**
```rust
// Before: Manual width calculation loop
let mut x_offset = 0u16;
for (i, menu) in menus.iter().enumerate() {
    let menu_width = display_width(menu) as u16 + 2;
    if col >= x_offset && col < x_offset + menu_width {
        // Found it
    }
    x_offset += menu_width + 2;
}

// After: Centralized function
if let Some(menu_idx) = find_clicked_menu(col, row, area, lang) {
    // Use it
}
```

## Menu Structure

```
File            (5 menus total)
├── Save Config         ⌘S
├── Load Config         ⌘O
├── ───────────         (separator)
└── Exit                ⌘Q

Session         (future: multi-session support)
├── New Session         ⌘T
├── Duplicate Session   ⌘D
├── Rename Session      F2
├── ───────────
└── Close Session       ⌘W

View            (future: layout support)
├── Single View
├── Split Horizontal
├── Split Vertical
├── Grid 2×2
├── ───────────
├── Next Pane           ⌘P
└── Previous Pane       ⇧⌘P

Settings
└── Toggle Language     ⇧⌘L

Help
├── Keyboard Shortcuts  F1
├── ───────────
└── About
```

## Benefits

### 1. Maintainability
- **One place to add/remove menu items**: Just update `menu_def.rs`
- **Compiler catches errors**: Exhaustive match on `MenuAction` enum
- **No index confusion**: Actions are named, not numbered

### 2. Type Safety
```rust
// Impossible to have wrong menu count
for i in 0..MENU_BAR.menu_count() {
    // Always correct
}

// Impossible to access non-existent item
match MENU_BAR.get_action(menu_idx, item_idx) {
    Some(action) => { /* handle */ }
    None => { /* can't happen with valid UI state */ }
}
```

### 3. Zero Runtime Cost
- All menu structures are `const` compile-time values
- No allocations, no runtime construction
- Function calls often inline to direct array access

### 4. Internationalization
- Menu labels use translation keys (`"menu.file"`, `"menu.file.save_config"`)
- Width calculations account for CJK characters automatically
- Consistent rendering across languages

## Testing

Added comprehensive tests in `menu_def.rs`:
```rust
#[test]
fn test_menu_bar_structure() {
    assert_eq!(MENU_BAR.menu_count(), 5);
    assert_eq!(MENU_BAR.get_item_count(0), 4);  // File menu
    assert_eq!(MENU_BAR.get_item_count(4), 3);  // Help menu
}

#[test]
fn test_menu_actions() {
    assert_eq!(MENU_BAR.get_action(0, 0), Some(MenuAction::SaveConfig));
    assert_eq!(MENU_BAR.get_action(99, 0), None);  // Invalid index
}
```

## Migration Guide

### Adding a New Menu Item

**Before:** Update 5+ different places with magic numbers

**After:** Two steps:
1. Add variant to `MenuAction` enum
2. Add to appropriate menu's `items` array

```rust
// Step 1: Add action
pub enum MenuAction {
    // ... existing
    ExportLogs,  // New!
}

impl MenuAction {
    pub fn label_key(&self) -> &'static str {
        match self {
            // ... existing
            MenuAction::ExportLogs => "menu.file.export_logs",
        }
    }
}

// Step 2: Add to menu
const FILE_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::SaveConfig,
    MenuAction::LoadConfig,
    MenuAction::ExportLogs,  // New!
    MenuAction::Separator,
    MenuAction::Exit,
];

// Step 3: Handle action (compiler forces you)
match action {
    // ... existing
    MenuAction::ExportLogs => {
        export_logs(app);
        false
    }
}
```

### Adding a New Menu

```rust
// 1. Add menu items
const NEW_MENU_ITEMS: &[MenuAction] = &[
    MenuAction::NewFeature1,
    MenuAction::NewFeature2,
];

// 2. Add to ALL_MENUS
const ALL_MENUS: &[Menu] = &[
    // ... existing menus
    Menu {
        label_key: "menu.new",
        items: NEW_MENU_ITEMS,
    },
];

// Done! Everything else updates automatically:
// - Menu bar rendering
// - Keyboard navigation
// - Mouse click detection
// - All count queries
```

## Keyboard Shortcuts

All shortcuts working correctly:

- **F10**: Open menu bar
- **←/→**: Navigate menus
- **↑/↓**: Navigate items
- **Enter**: Execute action
- **Esc**: Close menu
- **F1 or ?**: Toggle keyboard shortcuts help
- **⌘S**: Save config (works both in menu and as global shortcut)
- **⌘O**: Load config
- **⌘Q**: Quit

## Mouse Interaction

Fixed and working:

- **Click menu**: Open dropdown
- **Click item**: Execute action (ignores separators)
- **Click same menu**: Close dropdown
- **Click outside**: Close dropdown
- **Proper CJK character width**: Handles Chinese/Japanese menu labels

## Performance

- **Before**: Multiple string allocations per frame, O(n) menu searches
- **After**: Zero allocations, O(1) array access, compile-time constants

## Future Work

- Add session management actions (partially defined, awaiting multi-session feature)
- Add layout management actions (partially defined, awaiting layout feature)
- Consider adding keyboard accelerator display in menu items
- Add menu item enable/disable state (e.g., "Disconnect" only when connected)

## Conclusion

This refactoring follows Linus's principle: **fix the data structure, and the code fixes itself.**

By centralizing menu definitions, we:
- ✅ Eliminated all magic numbers
- ✅ Made the system type-safe
- ✅ Simplified maintenance
- ✅ Fixed all mouse interaction bugs
- ✅ Made adding new features trivial

**The code is now simpler, safer, and faster.**