//! Multi-tab and split-pane management for tuiserial
//!
//! This crate provides session management, layout management, and UI rendering
//! for displaying multiple serial port connections simultaneously. It supports:
//!
//! - Multiple serial port sessions with independent configurations
//! - Tab-based session switching
//! - Split-pane layouts (horizontal, vertical, grid)
//! - Session persistence and state management
//!
//! ## Architecture
//!
//! The tabs system is organized into modular components:
//! - `session`: Session management and state for multiple serial ports
//! - `layout`: Layout calculation and pane management for split views
//! - `tabs_ui`: UI rendering for tabs, panes, and session controls
//!
//! ## Usage
//!
//! ```rust,ignore
//! use tuiserial_tabs::{SessionManager, PaneManager, LayoutMode};
//!
//! // Create a session manager
//! let mut sessions = SessionManager::new();
//!
//! // Add more sessions
//! sessions.add_session(Some("COM1".to_string()));
//! sessions.add_session(Some("COM2".to_string()));
//!
//! // Create a pane manager for split views
//! let mut panes = PaneManager::new();
//! panes.set_layout_mode(LayoutMode::SplitHorizontal);
//!
//! // Switch between sessions
//! sessions.next_session();
//!
//! // Get active session
//! let active = sessions.active_session();
//! ```

// Module declarations
pub mod layout;
pub mod session;
pub mod tabs_ui;

// Re-exports for convenience
pub use layout::{LayoutMode, PaneManager};
pub use session::{SerialSession, SessionManager};

// Re-export UI rendering functions
pub use tabs_ui::{
    calculate_tab_bar_height, draw_compact_tab_bar, draw_layout_indicator, draw_pane_border,
    draw_session_info_overlay, draw_session_list, draw_tab_bar, draw_tab_bar_with_controls,
    draw_tab_notification_badge, get_tab_at_position,
};

// Re-export commonly used dependencies
pub use ratatui;
pub use tuiserial_core;

/// Version information for the tabs crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Combined manager for sessions and layout
///
/// This provides a unified interface for managing both sessions and their
/// layout/display configuration.
pub struct TabsManager {
    /// Session manager
    sessions: SessionManager,

    /// Pane/layout manager
    panes: PaneManager,

    /// Whether to show the tab bar
    show_tabs: bool,

    /// Whether to show layout controls
    show_layout_controls: bool,
}

impl TabsManager {
    /// Create a new tabs manager with default settings
    pub fn new() -> Self {
        Self {
            sessions: SessionManager::new(),
            panes: PaneManager::new(),
            show_tabs: true,
            show_layout_controls: true,
        }
    }

    /// Get the session manager
    pub fn sessions(&self) -> &SessionManager {
        &self.sessions
    }

    /// Get the session manager mutably
    pub fn sessions_mut(&mut self) -> &mut SessionManager {
        &mut self.sessions
    }

    /// Get the pane manager
    pub fn panes(&self) -> &PaneManager {
        &self.panes
    }

    /// Get the pane manager mutably
    pub fn panes_mut(&mut self) -> &mut PaneManager {
        &mut self.panes
    }

    /// Get the active session
    pub fn active_session(&self) -> &SerialSession {
        self.sessions.active_session()
    }

    /// Get the active session mutably
    pub fn active_session_mut(&mut self) -> &mut SerialSession {
        self.sessions.active_session_mut()
    }

    /// Get the session for the currently focused pane
    pub fn focused_pane_session(&self) -> Option<&SerialSession> {
        let session_idx = self.panes.focused_session()?;
        self.sessions.get_session(session_idx)
    }

    /// Get the session for the currently focused pane mutably
    pub fn focused_pane_session_mut(&mut self) -> Option<&mut SerialSession> {
        let session_idx = self.panes.focused_session()?;
        self.sessions.get_session_mut(session_idx)
    }

    /// Add a new session
    pub fn add_session(&mut self, name: Option<String>) -> usize {
        self.sessions.add_session(name)
    }

    /// Add a new session with a specific port
    pub fn add_session_with_port(&mut self, port: String, name: Option<String>) -> usize {
        self.sessions.add_session_with_port(port, name)
    }

    /// Remove a session
    pub fn remove_session(&mut self, index: usize) -> Option<SerialSession> {
        let removed = self.sessions.remove_session(index)?;

        // Update pane mappings if needed
        let total_sessions = self.sessions.len();
        for pane_idx in 0..self.panes.pane_count() {
            if let Some(session_idx) = self.panes.session_for_pane(pane_idx) {
                if session_idx >= total_sessions {
                    self.panes
                        .set_pane_session(pane_idx, total_sessions.saturating_sub(1));
                }
            }
        }

        Some(removed)
    }

    /// Switch to the next layout mode
    pub fn next_layout(&mut self) {
        self.panes.next_layout();

        // Ensure we have enough sessions for the new layout
        let required_sessions = self.panes.pane_count();
        let current_sessions = self.sessions.len();

        for i in current_sessions..required_sessions {
            self.sessions
                .add_session(Some(format!("Session {}", i + 1)));
        }
    }

    /// Switch to the previous layout mode
    pub fn prev_layout(&mut self) {
        self.panes.prev_layout();
    }

    /// Set whether to show the tab bar
    pub fn set_show_tabs(&mut self, show: bool) {
        self.show_tabs = show;
    }

    /// Check if the tab bar should be shown
    pub fn should_show_tabs(&self) -> bool {
        self.show_tabs && self.sessions.len() > 1
    }

    /// Set whether to show layout controls
    pub fn set_show_layout_controls(&mut self, show: bool) {
        self.show_layout_controls = show;
    }

    /// Check if layout controls should be shown
    pub fn should_show_layout_controls(&self) -> bool {
        self.show_layout_controls
    }

    /// Focus the next pane
    pub fn focus_next_pane(&mut self) {
        self.panes.focus_next_pane();
    }

    /// Focus the previous pane
    pub fn focus_prev_pane(&mut self) {
        self.panes.focus_prev_pane();
    }

    /// Switch to the next session in the focused pane
    pub fn cycle_focused_pane_session(&mut self) {
        let total_sessions = self.sessions.len();
        self.panes.cycle_focused_session(total_sessions);
    }

    /// Switch to the previous session in the focused pane
    pub fn cycle_focused_pane_session_prev(&mut self) {
        let total_sessions = self.sessions.len();
        self.panes.cycle_focused_session_prev(total_sessions);
    }

    /// Update all notifications for all sessions
    pub fn update_notifications(&mut self) {
        self.sessions.update_all_notifications();
    }

    /// Get the current layout mode
    pub fn layout_mode(&self) -> LayoutMode {
        self.panes.layout_mode()
    }

    /// Get the number of visible panes
    pub fn visible_pane_count(&self) -> usize {
        self.panes.pane_count()
    }

    /// Get session for a specific pane
    pub fn session_for_pane(&self, pane_index: usize) -> Option<&SerialSession> {
        let session_idx = self.panes.session_for_pane(pane_index)?;
        self.sessions.get_session(session_idx)
    }

    /// Get session for a specific pane mutably
    pub fn session_for_pane_mut(&mut self, pane_index: usize) -> Option<&mut SerialSession> {
        let session_idx = self.panes.session_for_pane(pane_index)?;
        self.sessions.get_session_mut(session_idx)
    }

    /// Check if a pane is focused
    pub fn is_pane_focused(&self, pane_index: usize) -> bool {
        self.panes.is_pane_focused(pane_index)
    }
}

impl Default for TabsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tabs_manager_creation() {
        let manager = TabsManager::new();
        assert_eq!(manager.sessions().len(), 1);
        assert_eq!(manager.panes().pane_count(), 1);
        assert_eq!(manager.layout_mode(), LayoutMode::Single);
    }

    #[test]
    fn test_add_remove_sessions() {
        let mut manager = TabsManager::new();

        let idx1 = manager.add_session(Some("Test 1".to_string()));
        let idx2 = manager.add_session(Some("Test 2".to_string()));

        assert_eq!(manager.sessions().len(), 3);

        manager.remove_session(idx1);
        assert_eq!(manager.sessions().len(), 2);
    }

    #[test]
    fn test_layout_switching() {
        let mut manager = TabsManager::new();

        assert_eq!(manager.layout_mode(), LayoutMode::Single);

        manager.next_layout();
        assert_eq!(manager.layout_mode(), LayoutMode::SplitHorizontal);
        assert_eq!(manager.visible_pane_count(), 2);

        manager.next_layout();
        assert_eq!(manager.layout_mode(), LayoutMode::SplitVertical);
    }

    #[test]
    fn test_pane_focus() {
        let mut manager = TabsManager::new();
        manager.next_layout(); // Switch to split mode

        assert_eq!(manager.panes().focused_pane(), 0);

        manager.focus_next_pane();
        assert_eq!(manager.panes().focused_pane(), 1);

        manager.focus_prev_pane();
        assert_eq!(manager.panes().focused_pane(), 0);
    }
}
