//! Serial port module
//!
//! Contains serial port drivers and wrappers:
//! - `wrapper`: Configuration wrapper for conditional serial output
//! - `uart_16550`: 16550 UART driver implementation

pub mod uart_16550;
pub mod wrapper;

// Re-export commonly used items
pub use uart_16550::SerialPort;
pub use wrapper::{disable, enable, init, is_enabled, write_byte_raw, write_bytes, write_str};
