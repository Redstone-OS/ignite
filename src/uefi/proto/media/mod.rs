//! Protocolos de Media

pub mod file;
pub mod fs;

// Re-export commonly used file types
pub use file::{CStr16, FileAttribute, FileMode, FileProtocol};
