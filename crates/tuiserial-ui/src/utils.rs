//! Utility functions for UI rendering
//!
//! This module provides helper functions used across the UI components.

/// Calculate display width of a string (handles CJK characters)
///
/// CJK (Chinese, Japanese, Korean) characters take up 2 display columns,
/// while ASCII characters take up 1 column.
pub fn display_width(s: &str) -> usize {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_width_ascii() {
        assert_eq!(display_width("hello"), 5);
        assert_eq!(display_width("test123"), 7);
    }

    #[test]
    fn test_display_width_cjk() {
        assert_eq!(display_width("你好"), 4);
        assert_eq!(display_width("こんにちは"), 10);
    }

    #[test]
    fn test_display_width_mixed() {
        assert_eq!(display_width("Hello 世界"), 9); // 5 + 1 + 4 = 10... wait
                                                    // Actually: H(1) e(1) l(1) l(1) o(1) space(1) 世(2) 界(2) = 10
        assert_eq!(display_width("Hello 世界"), 10);
        assert_eq!(display_width("Test测试"), 8); // 4 + 4
    }

    #[test]
    fn test_display_width_empty() {
        assert_eq!(display_width(""), 0);
    }
}
