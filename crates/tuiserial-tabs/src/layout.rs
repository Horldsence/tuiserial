//! Layout management for multiple sessions
//!
//! This module provides layout management for displaying multiple serial port sessions
//! simultaneously, supporting different split modes (horizontal, vertical, grid).
//!
//! The canonical `LayoutMode` type is defined in `tuiserial_core`.

use ratatui::layout::Rect;

// Re-export from core — single source of truth
pub use tuiserial_core::types::LayoutMode;

/// Pane manager for tracking visible panes and their mappings to sessions
pub struct PaneManager {
    /// Current layout mode
    layout_mode: LayoutMode,

    /// Mapping of pane index to session index
    pane_to_session: Vec<usize>,

    /// Currently focused pane index
    focused_pane: usize,
}

impl PaneManager {
    /// Create a new pane manager with default settings
    pub fn new() -> Self {
        Self {
            layout_mode: LayoutMode::Single,
            pane_to_session: vec![0],
            focused_pane: 0,
        }
    }

    /// Get the current layout mode
    pub fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    /// Set the layout mode
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
        self.adjust_panes();
    }

    /// Switch to the next layout mode
    pub fn next_layout(&mut self) {
        self.layout_mode = self.layout_mode.next();
        self.adjust_panes();
    }

    /// Switch to the previous layout mode
    pub fn prev_layout(&mut self) {
        self.layout_mode = self.layout_mode.prev();
        self.adjust_panes();
    }

    /// Get the number of visible panes
    pub fn pane_count(&self) -> usize {
        self.pane_to_session.len()
    }

    /// Get the focused pane index
    pub fn focused_pane(&self) -> usize {
        self.focused_pane
    }

    /// Get the session index for a specific pane
    pub fn session_for_pane(&self, pane_index: usize) -> Option<usize> {
        self.pane_to_session.get(pane_index).copied()
    }

    /// Get all pane-to-session mappings
    pub fn pane_mappings(&self) -> &[usize] {
        &self.pane_to_session
    }

    /// Set the session for a specific pane
    pub fn set_pane_session(&mut self, pane_index: usize, session_index: usize) {
        if pane_index < self.pane_to_session.len() {
            self.pane_to_session[pane_index] = session_index;
        }
    }

    /// Focus the next pane
    pub fn focus_next_pane(&mut self) {
        if !self.pane_to_session.is_empty() {
            self.focused_pane = (self.focused_pane + 1) % self.pane_to_session.len();
        }
    }

    /// Focus the previous pane
    pub fn focus_prev_pane(&mut self) {
        if !self.pane_to_session.is_empty() {
            if self.focused_pane == 0 {
                self.focused_pane = self.pane_to_session.len() - 1;
            } else {
                self.focused_pane -= 1;
            }
        }
    }

    /// Focus a specific pane
    pub fn focus_pane(&mut self, pane_index: usize) -> bool {
        if pane_index < self.pane_to_session.len() {
            self.focused_pane = pane_index;
            true
        } else {
            false
        }
    }

    /// Check if a pane is focused
    pub fn is_pane_focused(&self, pane_index: usize) -> bool {
        self.focused_pane == pane_index
    }

    /// Adjust panes when layout mode changes
    fn adjust_panes(&mut self) {
        let max_panes = self.layout_mode.max_panes();
        let current_panes = self.pane_to_session.len();

        if max_panes > current_panes {
            // Add more panes, mapping to consecutive sessions
            for i in current_panes..max_panes {
                self.pane_to_session.push(i);
            }
        } else if max_panes < current_panes {
            // Remove excess panes
            self.pane_to_session.truncate(max_panes);
        }

        // Ensure focused pane is valid
        if self.focused_pane >= self.pane_to_session.len() {
            self.focused_pane = self.pane_to_session.len().saturating_sub(1);
        }
    }

    /// Cycle the session in the focused pane to the next session
    pub fn cycle_focused_session(&mut self, total_sessions: usize) {
        if total_sessions == 0 {
            return;
        }
        if let Some(session_idx) = self.pane_to_session.get_mut(self.focused_pane) {
            *session_idx = (*session_idx + 1) % total_sessions;
        }
    }

    /// Cycle the session in the focused pane to the previous session
    pub fn cycle_focused_session_prev(&mut self, total_sessions: usize) {
        if total_sessions == 0 {
            return;
        }
        if let Some(session_idx) = self.pane_to_session.get_mut(self.focused_pane) {
            if *session_idx == 0 {
                *session_idx = total_sessions - 1;
            } else {
                *session_idx -= 1;
            }
        }
    }

    /// Get the session index for the focused pane
    pub fn focused_session(&self) -> Option<usize> {
        self.session_for_pane(self.focused_pane)
    }

    /// Calculate layout areas for the current mode
    pub fn calculate_areas(&self, area: Rect) -> Vec<Rect> {
        self.layout_mode.calculate_areas(area)
    }

    /// Reset pane mappings to sequential sessions
    pub fn reset_mappings(&mut self) {
        for (i, session_idx) in self.pane_to_session.iter_mut().enumerate() {
            *session_idx = i;
        }
    }
}

impl Default for PaneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_mode_cycle() {
        let mode = LayoutMode::Single;
        assert_eq!(mode.next(), LayoutMode::SplitHorizontal);
        assert_eq!(mode.prev(), LayoutMode::Grid2x1);
    }

    #[test]
    fn test_layout_max_panes() {
        assert_eq!(LayoutMode::Single.max_panes(), 1);
        assert_eq!(LayoutMode::SplitHorizontal.max_panes(), 2);
        assert_eq!(LayoutMode::Grid2x2.max_panes(), 4);
    }

    #[test]
    fn test_pane_manager_focus() {
        let mut manager = PaneManager::new();
        manager.set_layout_mode(LayoutMode::Grid2x2);

        assert_eq!(manager.pane_count(), 4);
        assert_eq!(manager.focused_pane(), 0);

        manager.focus_next_pane();
        assert_eq!(manager.focused_pane(), 1);

        manager.focus_prev_pane();
        assert_eq!(manager.focused_pane(), 0);
    }

    #[test]
    fn test_pane_manager_layout_change() {
        let mut manager = PaneManager::new();
        assert_eq!(manager.pane_count(), 1);

        manager.next_layout();
        assert_eq!(manager.layout_mode(), LayoutMode::SplitHorizontal);
        assert_eq!(manager.pane_count(), 2);

        manager.next_layout();
        assert_eq!(manager.layout_mode(), LayoutMode::SplitVertical);
        assert_eq!(manager.pane_count(), 2);
    }

    #[test]
    fn test_cycle_sessions() {
        let mut manager = PaneManager::new();
        manager.set_layout_mode(LayoutMode::SplitHorizontal);

        assert_eq!(manager.focused_session(), Some(0));
        manager.cycle_focused_session(3);
        assert_eq!(manager.focused_session(), Some(1));
        manager.cycle_focused_session(3);
        assert_eq!(manager.focused_session(), Some(2));
        manager.cycle_focused_session(3);
        assert_eq!(manager.focused_session(), Some(0));
    }
}
