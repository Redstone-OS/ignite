//! Abstração de Hardware
//!
//! ACPI, FDT, e outras interfaces de hardware

pub mod acpi;
pub mod fdt;
pub mod io;
pub mod serial;

pub use acpi::AcpiManager;
pub use fdt::DeviceTree;
pub use io::{Io, Mmio, Pio, ReadOnly};
