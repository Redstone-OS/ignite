//! Hardware Abstraction
//!
//! ACPI, FDT, and other hardware interfaces

pub mod acpi;
pub mod fdt;

pub use acpi::AcpiManager;
pub use fdt::DeviceTree;
