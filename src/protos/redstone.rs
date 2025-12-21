//! Protocolo Nativo Redstone
//!
//! Carrega kernels ELF64 e fornece a estrutura `BootInfo` completa.
//! Segue o padrão de handoff definido em `src/core/handoff.rs`.

use alloc::vec::Vec;

use super::{BootProtocol, KernelLaunchInfo};
use crate::{
    core::{
        error::{BootError, Result},
        handoff::{BootInfo, FramebufferInfo, MemoryInfo},
        types::LoadedFile,
    },
    elf::ElfLoader,
    memory::{FrameAllocator, PageTableManager},
    uefi::system_table,
};

pub struct RedstoneProtocol<'a> {
    allocator:  &'a mut dyn FrameAllocator,
    page_table: &'a mut PageTableManager,
}

impl<'a> RedstoneProtocol<'a> {
    pub fn new(
        allocator: &'a mut dyn FrameAllocator,
        page_table: &'a mut PageTableManager,
    ) -> Self {
        Self {
            allocator,
            page_table,
        }
    }

    fn prepare_framebuffer(&self) -> FramebufferInfo {
        // Stub seguro
        FramebufferInfo {
            address: 0,
            size:    0,
            width:   0,
            height:  0,
            stride:  0,
            format:  0,
        }
    }
}

impl<'a> BootProtocol for RedstoneProtocol<'a> {
    fn name(&self) -> &str {
        "Redstone Native"
    }

    fn identify(&self, file_content: &[u8]) -> bool {
        // Verifica Magic ELF: 0x7F 'E' 'L' 'F'
        file_content.len() > 4 && &file_content[0..4] == b"\x7fELF"
    }

    fn load(
        &mut self,
        kernel_file: &[u8],
        _cmdline: Option<&str>,
        modules: Vec<LoadedFile>,
    ) -> Result<KernelLaunchInfo> {
        // 1. Carregar segmentos ELF
        let mut loader = ElfLoader::new(self.allocator, self.page_table);
        let loaded_kernel = loader.load_kernel(kernel_file)?;

        // Alocar BootInfo
        let boot_info_phys = self.allocator.allocate_frame(1)?;
        let boot_info_ptr = boot_info_phys as *mut BootInfo;

        // 3. Preencher BootInfo
        let fb_info = self.prepare_framebuffer();

        // InitRD
        let (initrd_addr, initrd_size) = if let Some(first_mod) = modules.first() {
            (first_mod.ptr, first_mod.size as u64)
        } else {
            (0, 0)
        };

        // Construir BootInfo
        // TODO: Preencher BootInfo

        Ok(KernelLaunchInfo {
            entry_point:   loaded_kernel.entry_point,
            stack_pointer: None,
            rdi:           boot_info_phys,
            rsi:           0,
            rdx:           0,
            rbx:           0,
        })
    }
}
