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
            addr:   0,
            size:   0,
            width:  0,
            height: 0,
            stride: 0,
            format: crate::core::handoff::PixelFormat::Rgb,
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
        memory_map_buffer: (u64, u64),
    ) -> Result<KernelLaunchInfo> {
        // 1. Carregar segmentos ELF
        let mut loader = ElfLoader::new(self.allocator, self.page_table);
        let loaded_kernel = loader.load_kernel(kernel_file)?;

        // 2. CRÍTICO: Identity map dos primeiros 4GiB
        // Sem isso, o bootloader crasha ao trocar CR3 pois o código
        // do próprio bootloader fica inacessível!
        self.page_table
            .identity_map_4gib(self.allocator)
            .expect("Falha ao criar identity map");

        // 3. Alocar BootInfo
        let boot_info_phys = self.allocator.allocate_frame(1)?;
        let boot_info_ptr = boot_info_phys as *mut BootInfo;

        // 4. Preencher BootInfo
        let fb_info = self.prepare_framebuffer();

        // InitRD
        let (initrd_addr, initrd_size) = if let Some(first_mod) = modules.first() {
            (first_mod.ptr, first_mod.size as u64)
        } else {
            (0, 0)
        };

        // Construir BootInfo e escrever na memória alocada
        let boot_info = BootInfo {
            magic:   crate::core::handoff::BOOT_INFO_MAGIC,
            version: crate::core::handoff::BOOT_INFO_VERSION,

            framebuffer: fb_info,

            memory_map_addr: memory_map_buffer.0, // Ponteiro do memory map
            memory_map_len:  memory_map_buffer.1, // Contagem de entradas

            rsdp_addr: 0, // TODO: Buscar RSDP do ACPI

            kernel_phys_addr: loaded_kernel.base_address,
            kernel_size:      loaded_kernel.size,

            initramfs_addr: initrd_addr,
            initramfs_size: initrd_size,
        };

        // CRÍTICO: Escrever o BootInfo no ponteiro alocado
        unsafe {
            core::ptr::write(boot_info_ptr, boot_info);
        }

        Ok(KernelLaunchInfo {
            entry_point: loaded_kernel.entry_point,
            use_fixed_redstone_entry: true, // Protocolo Redstone usa jump fixo
            stack_pointer: None,
            rdi: boot_info_phys,
            rsi: 0,
            rdx: 0,
            rbx: 0,
        })
    }
}
