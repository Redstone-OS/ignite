//! Núcleo do Bootloader (Core)
//!
//! Contém as definições fundamentais, tratamento de erros, configuração e
//! estruturas de handoff para o kernel. Este módulo não deve depender de
//! drivers específicos ou UEFI complexo.

pub mod config;
pub mod error;
pub mod handoff;
pub mod logging;

// Re-exports para facilitar o acesso
pub use config::meta;
pub use error::{BootError, Result};
pub use handoff::BootInfo;
