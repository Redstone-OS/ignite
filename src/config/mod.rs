//! Módulo de configuração
//!
//! Gerencia configurações do bootloader via arquivo de configuração

pub mod boot_config;

pub use boot_config::{BootConfig, BootMenuConfig, OsEntry};
