//! Input Handling
//!
//! Keyboard and mouse input processing

/// Key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Char(char),
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Input handler
pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    /// Wait for key press (blocking)
    pub fn wait_key(&self) -> Option<Key> {
        // TODO: Implement actual key reading from UEFI
        // For now, return None
        None
    }

    /// Check if key is available (non-blocking)
    pub fn key_available(&self) -> bool {
        // TODO: Implement
        false
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
