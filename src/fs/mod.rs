//! Subsistema de Arquivos
//!
//! Fornece acesso unificado a arquivos via trait `VFS`.
//! Suporta o sistema de arquivos de boot (UEFI) e futuramente drivers nativos.

pub mod dev;
pub mod fat32;
pub mod loader;
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

/// Lê exatamente `buffer.len()` bytes do arquivo para o buffer fornecido.
/// Retorna erro se não conseguir ler todos os bytes (arquivo truncado ou corrompido).
/// 
/// Esta função é utilizada quando você já alocou memória (ex: via UEFI allocate_pool)
/// e quer ler o arquivo diretamente neste buffer, sem alocações intermediárias.
pub fn read_exact(file: &mut dyn File, buffer: &mut [u8]) -> crate::core::error::Result<()> {
    let mut total_read = 0;
    
    while total_read < buffer.len() {
        let n = file.read(&mut buffer[total_read..])?;
        
        if n == 0 {
            // EOF antes de ler tudo - arquivo corrompido ou menor que esperado
            return Err(crate::core::error::BootError::FileSystem(
                crate::core::error::FileSystemError::ReadError
            ));
        }
        
        total_read += n;
    }
    
    Ok(())
}
