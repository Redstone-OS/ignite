//! Abstração de Hardware
//!
//! Fornece drivers e interfaces para interagir com o hardware físico e virtual.
//! Serve como a ponte entre o sistema de arquivos/core e a
//! arquitetura/firmware.

pub mod acpi;
pub mod io;
pub mod serial;
pub mod storage;

// Re-exports
pub use io::Mmio;
pub use serial::SerialPort;
pub use storage::UefiBlockDevice;
