//! Protocolos de Mídia (Disco, Arquivos)

pub mod file;
pub mod fs;

// Re-exports
pub use file::FileProtocol;
pub use fs::SimpleFileSystemProtocol;
