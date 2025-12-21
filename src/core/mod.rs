//! Núcleo do Bootloader (Core)
//!
//! Contém as definições fundamentais, tratamento de erros, configuração e
//! estruturas de handoff para o kernel. Este módulo não deve depender de
//! drivers específicos ou UEFI complexo.

pub mod config;
pub mod error;
pub mod handoff;
pub mod logging;
pub mod types; // Expondo o módulo types.rs

// Re-exports para facilitar o acesso
pub use config::meta;
pub use error::{BootError, Result};
pub use handoff::BootInfo;
// Re-exportar tipos comuns para facilitar o uso (ex: crate::core::LoadedFile)
pub use types::{Framebuffer, LoadedFile, LoadedKernel};
