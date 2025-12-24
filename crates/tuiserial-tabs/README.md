# tuiserial-tabs

Multi-tab and split-pane management for tuiserial serial port monitor.

## Features

- 📑 **Multiple Sessions**: Manage multiple serial port connections simultaneously
- 🔀 **Flexible Layouts**: Switch between single, split, and grid layouts
- 🎯 **Independent State**: Each session maintains its own configuration and logs
- 🎨 **Rich UI**: Tab bar with connection indicators and session names
- ⌨️ **Keyboard Navigation**: Full keyboard support for session and pane management
- 🖱️ **Mouse Support**: Click tabs to switch sessions

## Architecture

This crate is organized into three main modules:

### `session`
Manages individual serial port sessions and the session manager:
- `SerialSession`: Individual session state (config, logs, UI state)
- `SessionManager`: Manages multiple sessions with switching and lifecycle

### `layout`
Handles layout calculation and pane management:
- `LayoutMode`: Different layout modes (Single, Split, Grid)
- `PaneManager`: Manages visible panes and their session mappings

### `tabs_ui`
UI rendering functions for tabs and panes:
- Tab bar rendering with connection indicators
- Pane border rendering with focus highlights
- Session overlay dialogs
- Layout mode indicators

## Usage

### Basic Session Management

```rust
use tuiserial_tabs::{SessionManager, SerialSession};

// Create a session manager (starts with one default session)
let mut sessions = SessionManager::new();

// Add more sessions
let idx1 = sessions.add_session(Some("COM1".to_string()));
let idx2 = sessions.add_session_with_port("COM3".to_string(), None);

// Switch between sessions
sessions.next_session();
sessions.prev_session();
sessions.switch_to(idx1);

// Get active session
let active = sessions.active_session_mut();
active.config.baud_rate = 115200;

// Remove a session
sessions.remove_session(idx2);
```

### Layout Management

```rust
use tuiserial_tabs::{PaneManager, LayoutMode};

// Create a pane manager
let mut panes = PaneManager::new();

// Switch layout modes
panes.set_layout_mode(LayoutMode::SplitHorizontal);
panes.next_layout(); // Cycles through available layouts

// Focus management
panes.focus_next_pane();
panes.focus_prev_pane();
panes.focus_pane(0);

// Get session for focused pane
if let Some(session_idx) = panes.focused_session() {
    println!("Focused pane shows session {}", session_idx);
}

// Cycle sessions in focused pane
panes.cycle_focused_session(total_sessions);
```

### Unified Management

```rust
use tuiserial_tabs::TabsManager;

// Create a unified manager
let mut manager = TabsManager::new();

// Add sessions
manager.add_session_with_port("COM1".to_string(), Some("Arduino".to_string()));
manager.add_session_with_port("COM3".to_string(), Some("ESP32".to_string()));

// Switch layouts
manager.next_layout(); // Now shows both sessions in split view

// Navigate panes
manager.focus_next_pane();

// Get session for focused pane
if let Some(session) = manager.focused_pane_session_mut() {
    session.add_info("Message sent");
}
```

### UI Rendering

```rust
use ratatui::Frame;
use tuiserial_tabs::{draw_tab_bar, draw_pane_border, TabsManager};

fn render(f: &mut Frame, manager: &TabsManager) {
    let area = f.area();
    
    // Draw tab bar at top
    let tab_height = 3;
    let tab_area = Rect { x: 0, y: 0, width: area.width, height: tab_height };
    draw_tab_bar(f, tab_area, manager.sessions(), true);
    
    // Calculate pane areas
    let content_area = Rect { 
        x: 0, 
        y: tab_height, 
        width: area.width, 
        height: area.height - tab_height 
    };
    let pane_areas = manager.panes().calculate_areas(content_area);
    
    // Draw each pane
    for (pane_idx, pane_area) in pane_areas.iter().enumerate() {
        if let Some(session) = manager.session_for_pane(pane_idx) {
            let is_focused = manager.is_pane_focused(pane_idx);
            let inner = draw_pane_border(
                f, 
                *pane_area, 
                &session.name, 
                is_focused, 
                session.is_connected
            );
            
            // Draw session content in inner area...
        }
    }
}
```

## Layout Modes

### `Single`
- Shows one session at a time with tab bar for switching
- Default mode, most screen space for one session

### `SplitHorizontal`
- Shows two sessions, one on top, one on bottom
- 50/50 split

### `SplitVertical`
- Shows two sessions side by side
- 50/50 split

### `Grid2x2`
- Shows four sessions in a 2×2 grid
- Equal space for all sessions

### `Grid1x2`
- Shows three sessions: one large on top, two smaller on bottom
- Top: 50%, Bottom left: 25%, Bottom right: 25%

### `Grid2x1`
- Shows three sessions: one large on left, two smaller stacked on right
- Left: 50%, Top right: 25%, Bottom right: 25%

## Keyboard Shortcuts

When integrating with your application, suggested keyboard shortcuts:

- `Ctrl+T`: New session
- `Ctrl+W`: Close current session
- `Ctrl+Tab` or `Ctrl+→`: Next session/tab
- `Ctrl+Shift+Tab` or `Ctrl+←`: Previous session/tab
- `Ctrl+1-9`: Switch to session by number
- `Ctrl+L`: Cycle layout mode
- `Ctrl+Shift+L`: Previous layout mode
- `Ctrl+P`: Focus next pane
- `Ctrl+Shift+P`: Focus previous pane
- `F2`: Rename current session
- `Ctrl+D`: Duplicate current session

## Session State

Each `SerialSession` maintains:

- **Configuration**: Port, baud rate, data bits, parity, stop bits, flow control
- **Message Log**: Received and sent data with timestamps
- **UI State**: Dropdown selections, scroll position, focus
- **TX State**: Input buffer, cursor position, transmission mode
- **Connection State**: Connected/disconnected status
- **Notifications**: Session-specific notification queue

## Integration Example

```rust
use tuiserial_tabs::{TabsManager, LayoutMode};
use tuiserial_core::AppState;

struct App {
    tabs: TabsManager,
    // ... other state
}

impl App {
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('t') if self.modifiers.contains(Ctrl) => {
                // New session
                self.tabs.add_session(None);
            }
            KeyCode::Char('w') if self.modifiers.contains(Ctrl) => {
                // Close current session
                let active_idx = self.tabs.sessions().active_index();
                self.tabs.remove_session(active_idx);
            }
            KeyCode::Tab if self.modifiers.contains(Ctrl) => {
                // Next session
                self.tabs.sessions_mut().next_session();
            }
            KeyCode::Char('l') if self.modifiers.contains(Ctrl) => {
                // Cycle layout
                self.tabs.next_layout();
            }
            KeyCode::Char('p') if self.modifiers.contains(Ctrl) => {
                // Focus next pane
                self.tabs.focus_next_pane();
            }
            _ => {}
        }
    }
}
```

## Dependencies

- `tuiserial-core`: Core data models and types
- `ratatui`: Terminal UI framework
- `serde`: Serialization (for session persistence)
- `serde_json`: JSON serialization

## License

This crate is part of the tuiserial project and shares the same license.