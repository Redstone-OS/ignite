//! Chainloader EFI
//!
//! Permite carregar outros arquivos .EFI (ex: Windows Boot Manager, GRUB).
//! Utiliza a infraestrutura de `LoadImage` e `StartImage` do UEFI.

use alloc::vec::Vec;

use super::{BootProtocol, KernelLaunchInfo};
use crate::core::{error::Result, types::LoadedFile};

pub struct ChainloadProtocol;

impl BootProtocol for ChainloadProtocol {
    fn name(&self) -> &str {
        "EFI Chainload"
    }

    fn identify(&self, file_content: &[u8]) -> bool {
        // MZ signature (PE/COFF)
        file_content.len() > 2 && file_content[0] == b'M' && file_content[1] == b'Z'
    }

    fn load(
        &mut self,
        _kernel_file: &[u8],
        _cmdline: Option<&str>,
        _modules: Vec<LoadedFile>,
        _memory_map_buffer: (u64, u64),
    ) -> Result<KernelLaunchInfo> {
        // O Chainload em UEFI é especial: ele não retorna LaunchInfo para um salto
        // manual. Ele usa BS->LoadImage e BS->StartImage.
        // A arquitetura atual de LaunchInfo assume salto em Assembly.
        // Para chainload, executaríamos aqui e nunca retornaríamos (ou retornaríamos
        // erro).

        // TODO: Implementar usando uefi::boot_services().load_image()
        Err(crate::core::error::BootError::Generic(
            "Chainload deve ser executado diretamente",
        ))
    }
}
