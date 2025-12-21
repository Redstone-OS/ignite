//! Módulo de sistema de arquivos
//!
//! Responsável por carregar arquivos do sistema de arquivos UEFI

pub mod compression;
pub mod initfs;
pub mod loader; // Suporte a compressão de arquivos

pub use compression::{CompressionType, decompress, decompress_auto};
pub use initfs::InitFsLoader;
pub use loader::FileLoader;
