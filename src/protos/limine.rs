//! Limine Boot Protocol Implementation
//!
//! This is the native protocol for Redstone OS, compatible with the Limine
//! specification. https://codeberg.org/Limine/limine-protocol

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

/// Limine protocol implementation (native Redstone OS protocol)
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
        // Validate ELF header
        if kernel.len() < 64 {
            return Err(BootError::Generic("Kernel file too small"));
        }

        // Check ELF magic
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
        // For now, use the existing ELF loader
        // In a complete implementation, this would parse Limine requests/responses
        use crate::elf::ElfLoader;

        let elf_loader = ElfLoader::new(self.allocator);
        let loaded_kernel = elf_loader.load(kernel)?;

        self.entry_point = loaded_kernel.entry_point;
        self.kernel_base = loaded_kernel.base_address;
        self.kernel_size = loaded_kernel.size;

        info!("Limine protocol: Kernel loaded at {:#x}", self.kernel_base);

        // TODO: Create Limine boot information structure
        // This should include:
        // - Memory map
        // - Framebuffer info
        // - Modules
        // - RSDP pointer
        // - etc.

        Ok(BootInfo {
            entry_point:   self.entry_point,
            kernel_base:   self.kernel_base,
            kernel_size:   self.kernel_size,
            stack_ptr:     None, // Kernel manages its own stack
            boot_info_ptr: 0,    // TODO: Allocate and fill Limine boot info
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
