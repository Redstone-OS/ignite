//! # Bootloader Filesystem (VFS)
//!
//! Este módulo abstrai o acesso ao disco, permitindo que o Ignite carregue
//! arquivos (kernel, config, initrd) sem saber se estão em FAT32, NTFS ou
//! RedstoneFS.
//!
//! ## 🎯 Arquitetura VFS
//! O Ignite define traits simples (`File`, `Directory`, `FileSystem`) que os
//! drivers implementam. Atualmente, o driver principal é o `UefiFileSystem`,
//! que delega tudo para o firmware (via Simple File System Protocol).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Abstração:** Permite trocar o backend (ex: ler de Rede/PXE) sem mudar o
//!   `main.rs`.
//! - **Zero-Copy Friendly:** A função `read_exact` permite ler do disco direto
//!   para um buffer alocado pelo caller.
//!
//! ### ⚠️ Pontos de Atenção (Memory Fragmentation)
//! - **Alloc-Heavy Helpers:** `read_to_string` e `read_to_bytes` criam
//!   `Vec<u8>` dinamicamente.
//!   - *Risco:* Heap Fragmentation severa ao carregar arquivos grandes (Initrd
//!     > 100MB).
//!   - *Mitigação:* Use sempre `read_exact` com buffer pré-alocado
//!     (`allocate_pool`) para payloads grandes.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Driver)** Implementar driver **RedstoneFS Read-Only**.
//!   - *Meta:* Permitir que o `/boot` resida dentro do pool RFS, eliminando a
//!     dependência da partição ESP (FAT32) para o Kernel.

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
/// Retorna erro se não conseguir ler todos os bytes (arquivo truncado ou
/// corrompido).
///
/// Esta função é utilizada quando você já alocou memória (ex: via UEFI
/// allocate_pool) e quer ler o arquivo diretamente neste buffer, sem alocações
/// intermediárias.
pub fn read_exact(file: &mut dyn File, buffer: &mut [u8]) -> crate::core::error::Result<()> {
    let mut total_read = 0;

    while total_read < buffer.len() {
        let n = file.read(&mut buffer[total_read..])?;

        if n == 0 {
            // EOF antes de ler tudo - arquivo corrompido ou menor que esperado
            return Err(crate::core::error::BootError::FileSystem(
                crate::core::error::FileSystemError::ReadError,
            ));
        }

        total_read += n;
    }

    Ok(())
}
