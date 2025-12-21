//! Abstração de Hardware
//!
//! ACPI, FDT, e outras interfaces de hardware

pub mod acpi;
pub mod fdt;

pub use acpi::AcpiManager;
pub use fdt::DeviceTree;
