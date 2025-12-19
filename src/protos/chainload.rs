//! Chainloading Support
//!
//! Allows boot loading another bootloader (EFI applications or BIOS boot
//! sectors).

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

/// EFI chainload protocol
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

    /// Validate PE/COFF header
    fn validate_pe(&self, image: &[u8]) -> Result<()> {
        if image.len() < 64 {
            return Err(BootError::Generic("Image too small"));
        }

        // Check DOS header magic "MZ"
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
        // Load PE/COFF image
        // TODO: Parse PE headers and relocate image
        // For now, just load it into memory

        let pages = (image.len() + 4095) / 4096;
        let load_addr = self.allocator.allocate_any(pages)?;

        unsafe {
            core::ptr::copy_nonoverlapping(image.as_ptr(), load_addr as *mut u8, image.len());
        }

        // TODO: Parse PE to find entry point
        // For now, assume entry at start
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

/// BIOS chainload protocol
pub struct BiosChainloadProtocol {
    // BIOS chainloading is platform-specific and would only work on BIOS systems
    // This is a placeholder for when BIOS support is added
}

impl BiosChainloadProtocol {
    pub fn new() -> Self {
        Self {}
    }
}

// Note: This won't compile on UEFI-only builds, which is intentional
// #[cfg(target_os = "bios")]
// impl BootProtocol for BiosChainloadProtocol {
//     // Implementation for BIOS chainloading
// }
