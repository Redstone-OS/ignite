//! Módulo de sistema de arquivos
//!
//! Responsável por carregar arquivos do sistema de arquivos UEFI

pub mod loader;
pub mod initfs;

pub use loader::FileLoader;
pub use initfs::InitFsLoader;
