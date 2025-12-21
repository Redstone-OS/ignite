//! Módulo UEFI - Implementação Pura
//!
//! Implementação completa do UEFI baseada na especificação oficial 2.10,
//! sem dependências externas (apenas core Rust).
//!
//! Referência: https://uefi.org/specs/UEFI/2.10/

pub mod base;
pub mod helpers; // Helper functions for global state
pub mod proto;
pub mod table;

// Re-exports principais
pub use base::{Boolean, Char16, Event, FALSE, Guid, Handle, Result, Status, TRUE};
pub use table::{boot::BootServices, system::SystemTable};
