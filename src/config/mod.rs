//! Subsistema de Configuração
//!
//! Gerencia o carregamento, parsing e interpretação das opções de boot.

pub mod loader;
pub mod macros;
pub mod parser;
pub mod path;
pub mod types;

// Re-exports principais
pub use loader::load_configuration;
pub use path::ConfigPath;
pub use types::{BootConfig, Entry, Protocol};
