//! Tabelas UEFI

pub mod boot;
pub mod boot_helpers; // Helper methods for BootServices
pub mod runtime;
pub mod system;

pub use boot::BootServices;
pub use runtime::RuntimeServices;
pub use system::SystemTable;
