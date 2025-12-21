//! Sistema de Configuração
//!
//! Analisa arquivos de configuração de boot (ignite.conf / boot.cfg)

pub mod macros;
pub mod parser;
pub mod paths;
pub mod types;
pub mod validator;

pub use parser::ConfigParser;
pub use paths::{Path, PathResource};
pub use types::{BootConfig, MenuEntry, Module};
