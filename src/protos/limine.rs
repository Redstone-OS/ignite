//! Implementação do Protocolo de Boot Limine
//!
//! Este é o protocolo nativo para o Redstone OS, compatível com a especificação
//! Limine. https://codeberg.org/Limine/limine-protocol

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    core::error::{BootError, Result},
    memory::MemoryAllocator,
    core::types::LoadedFile,
};

/// Implementação do protocolo Limine (protocolo nativo do Redstone OS)
pub struct LimineProtocol<'a> {
    allocator:   &'a MemoryAllocator<'a>,
    entry_point: u64,
    kernel_base: u64,
    kernel_size: u64,
}

impl<'a> LimineProtocol<'a> {
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self {
            allocator,
            entry_point: 0,
            kernel_base: 0,
            kernel_size: 0,
        }
    }
}

impl<'a> BootProtocol for LimineProtocol<'a> {
    fn validate(&self, kernel: &[u8]) -> Result<()> {
        // Validar cabeçalho ELF
        if kernel.len() < 64 {
            return Err(BootError::Generic("Kernel file too small"));
        }

        // Checar magic ELF
        if &kernel[0..4] != b"\x7fELF" {
            return Err(BootError::Generic("Invalid ELF magic"));
        }

        info!("Limine protocol: ELF validation passed");
        Ok(())
    }

    fn prepare(
        &mut self,
        kernel: &[u8],
        _cmdline: Option<&str>,
        _modules: &[LoadedFile],
    ) -> Result<BootInfo> {
        // Por enquanto, usar o carregador ELF existente
        // Em uma implementação completa, isso analisaria requisições/respostas Limine
        use crate::elf::ElfLoader;

        let elf_loader = ElfLoader::new(self.allocator);
        let loaded_kernel = elf_loader.load(kernel)?;

        self.entry_point = loaded_kernel.entry_point;
        self.kernel_base = loaded_kernel.base_address;
        self.kernel_size = loaded_kernel.size;

        info!("Limine protocol: Kernel loaded at {:#x}", self.kernel_base);

        // TODO: Criar estrutura de informações de boot Limine
        // Isso deve incluir:
        // - Mapa de memória
        // - Info de Framebuffer
        // - Módulos
        // - Ponteiro RSDP
        // - etc.

        Ok(BootInfo {
            entry_point:   self.entry_point,
            kernel_base:   self.kernel_base,
            kernel_size:   self.kernel_size,
            stack_ptr:     None, // Kernel gerencia sua própria pilha
            boot_info_ptr: 0,    // TODO: Alocar e preencher info de boot Limine
            registers:     ProtocolRegisters::default(),
        })
    }

    fn entry_point(&self) -> u64 {
        self.entry_point
    }

    fn name(&self) -> &'static str {
        "limine"
    }
}

