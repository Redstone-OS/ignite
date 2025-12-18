//! Módulo de parsing e carregamento de arquivos ELF
//!
//! Responsável por parsear arquivos ELF e carregar seus segmentos na memória

pub mod parser;
pub mod loader;

pub use parser::ElfParser;
pub use loader::ElfLoader;
