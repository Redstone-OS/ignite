//! Suporte a Chainloading
//!
//! Permite carregar outro bootloader (aplicações EFI ou setores de boot BIOS).

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    core::error::{BootError, Result},
    memory::MemoryAllocator,
    core::types::LoadedFile,
};

/// Protocolo de chainload EFI
pub struct EfiChainloadProtocol<'a> {
    allocator:   &'a MemoryAllocator<'a>,
    entry_point: u64,
}

impl<'a> EfiChainloadProtocol<'a> {
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self {
            allocator,
            entry_point: 0,
        }
    }

    /// Validar cabeçalho PE/COFF
    fn validate_pe(&self, image: &[u8]) -> Result<()> {
        if image.len() < 64 {
            return Err(BootError::Generic("Image too small"));
        }

        // Checar magic do cabeçalho DOS "MZ"
        if &image[0..2] != b"MZ" {
            return Err(BootError::Generic("Invalid PE/COFF magic"));
        }

        info!("EFI chainload: PE/COFF validation passed");
        Ok(())
    }
}

impl<'a> BootProtocol for EfiChainloadProtocol<'a> {
    fn validate(&self, image: &[u8]) -> Result<()> {
        self.validate_pe(image)
    }

    fn prepare(
        &mut self,
        image: &[u8],
        _cmdline: Option<&str>,
        _modules: &[LoadedFile],
    ) -> Result<BootInfo> {
        // Carregar imagem PE/COFF
        // TODO: Analisar cabeçalhos PE e realocar imagem
        // Por enquanto, apenas carregar na memória

        let pages = (image.len() + 4095) / 4096;
        let load_addr = self.allocator.allocate_any(pages)?;

        unsafe {
            core::ptr::copy_nonoverlapping(image.as_ptr(), load_addr as *mut u8, image.len());
        }

        // TODO: Analisar PE para encontrar ponto de entrada
        // Por enquanto, assumir entrada no início
        self.entry_point = load_addr;

        info!("EFI chainload loaded at {:#x}", load_addr);

        Ok(BootInfo {
            entry_point:   self.entry_point,
            kernel_base:   load_addr,
            kernel_size:   image.len() as u64,
            stack_ptr:     None,
            boot_info_ptr: 0,
            registers:     ProtocolRegisters::default(),
        })
    }

    fn entry_point(&self) -> u64 {
        self.entry_point
    }

    fn name(&self) -> &'static str {
        "efi_chainload"
    }
}

/// Protocolo de chainload BIOS
pub struct BiosChainloadProtocol {
    // Chainloading BIOS é específico de plataforma e funcionaria apenas em sistemas BIOS
    // Este é um placeholder para quando o suporte a BIOS for adicionado
}

impl BiosChainloadProtocol {
    pub fn new() -> Self {
        Self {}
    }
}

// Nota: Isso não compilará em builds apenas UEFI, o que é intencional
// #[cfg(target_os = "bios")]
// impl BootProtocol for BiosChainloadProtocol {
//     // Implementação para chainloading BIOS
// }

