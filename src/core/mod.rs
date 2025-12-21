//! Core module - Fundamental utilities and types
//!
//! This module contains essential bootloader components including:
//! - Boot information structures
//! - Constants and configuration
//! - Error handling
//! - Type definitions
//! - Logging infrastructure

pub mod boot_info;
pub mod constants;
pub mod error;
pub mod logger;
pub mod types;

// Re-export commonly used items for convenience
pub use boot_info::BootInfo;
pub use error::{BootError, Result};
pub use types::*;
