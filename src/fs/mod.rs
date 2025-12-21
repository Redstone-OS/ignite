//! Subsistema de Arquivos
//!
//! Fornece acesso unificado a arquivos via trait `VFS`.
//! Suporta o sistema de arquivos de boot (UEFI) e futuramente drivers nativos.

pub mod dev;
pub mod fat32;
pub mod path;
pub mod redstonefs;
pub mod uefi;
pub mod vfs;

// Re-exports
pub use uefi::UefiFileSystem;
pub use vfs::{Directory, File, FileSystem, Metadata};

/// Helper para carregar um arquivo inteiro na memória (Vec<u8>).
/// Útil para carregar Kernel e Initrd.
pub fn read_to_string(file: &mut dyn File) -> crate::core::error::Result<alloc::string::String> {
    let mut buf = alloc::vec::Vec::new();
    let mut temp = [0u8; 1024];
    loop {
        let n = file.read(&mut temp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&temp[..n]);
    }
    alloc::string::String::from_utf8(buf)
        .map_err(|_| crate::core::error::BootError::Generic("Invalid UTF-8"))
}

pub fn read_to_bytes(file: &mut dyn File) -> crate::core::error::Result<alloc::vec::Vec<u8>> {
    let mut buf = alloc::vec::Vec::new();
    let mut temp = [0u8; 4096];
    loop {
        let n = file.read(&mut temp)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&temp[..n]);
    }
    Ok(buf)
}
