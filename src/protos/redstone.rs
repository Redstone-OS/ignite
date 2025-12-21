//! Protocolo Nativo Redstone
//!
//! Carrega kernels ELF64 e fornece a estrutura `BootInfo` completa.
//! Segue o padrão de handoff definido em `src/core/handoff.rs`.

use alloc::vec::Vec;

use super::{BootProtocol, KernelLaunchInfo};
use crate::{
    core::{
        error::{BootError, Result},
        handoff::{
            BOOT_INFO_MAGIC, BOOT_INFO_VERSION, BootInfo, FramebufferInfo, KernelInfo, MemoryInfo,
        },
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

    /// Prepara a estrutura de vídeo para o Kernel.
    fn prepare_framebuffer(&self) -> FramebufferInfo {
        // Tenta obter o driver de vídeo atual
        // Em um cenário real, o driver de vídeo deve armazenar seu estado em um local
        // global ou ser passado via contexto. Aqui, assumimos que podemos
        // consultar via UEFI GOP ou que o vídeo já foi inicializado.

        // Placeholder seguro:
        use crate::uefi::proto::console::gop::GRAPHICS_OUTPUT_PROTOCOL_GUID;
        let st = crate::uefi::system_table();

        // Tentativa simplificada de obter info do GOP (se disponível)
        if let Ok(gop_ptr) = st
            .boot_services()
            .locate_protocol(&GRAPHICS_OUTPUT_PROTOCOL_GUID)
        {
            let gop = unsafe {
                &*(gop_ptr as *const crate::uefi::proto::console::gop::GraphicsOutputProtocol)
            };
            let mode = unsafe { &*gop.mode };
            let info = unsafe { &*mode.info };

            FramebufferInfo {
                address: mode.frame_buffer_base,
                size:    mode.frame_buffer_size,
                width:   info.horizontal_resolution,
                height:  info.vertical_resolution,
                stride:  info.pixels_per_scan_line,
                format:  info.pixel_format as u32,
            }
        } else {
            // Fallback ou erro
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

        // 2. Preparar BootInfo (Alocar na Heap ou Páginas)
        // Precisamos alocar memória que o kernel possa ler.
        // Usamos o alocador de páginas para garantir persistência e endereço físico
        // conhecido.
        let boot_info_pages = 1; // 4KiB é suficiente para a struct BootInfo
        let boot_info_phys = self.allocator.allocate_frame(boot_info_pages)?;

        // Mapear temporariamente para escrita (Identity map já deve existir para essa
        // região)
        let boot_info_ptr = boot_info_phys as *mut BootInfo;

        // 3. Preencher BootInfo
        let fb_info = self.prepare_framebuffer();

        let kernel_info = KernelInfo {
            phys_addr:   loaded_kernel.base_address,
            entry_point: loaded_kernel.entry_point,
            size:        loaded_kernel.size,
            stack_base:  0, // Stack será definida pelo loader ou kernel
            stack_size:  0,
        };

        // InitRD
        let (initrd_addr, initrd_size) = if let Some(first_mod) = modules.first() {
            (first_mod.ptr, first_mod.size as u64)
        } else {
            (0, 0)
        };

        let boot_info = BootInfo {
            magic: BOOT_INFO_MAGIC,
            version: BOOT_INFO_VERSION,
            memory: MemoryInfo {
                map_addr:       0, // TODO: Copiar mapa de memória UEFI
                map_count:      0,
                page_table_cr3: self.page_table.pml4_addr(),
            },
            framebuffer: fb_info,
            kernel: kernel_info,
            initrd_addr,
            initrd_size,
            rsdp_addr: 0, // TODO: Localizar via SystemTable
            uefi_system_table: system_table() as *const _ as u64,
        };

        // Escrever na memória
        unsafe {
            boot_info_ptr.write(boot_info);
        }

        // 4. Retornar Info de Lançamento
        Ok(KernelLaunchInfo {
            entry_point:   loaded_kernel.entry_point,
            stack_pointer: None, // Kernel define sua própria stack ou usamos valor padrão
            rdi:           boot_info_phys, // RDI = Ponteiro para BootInfo (Convenção System V)
            rsi:           0,
            rdx:           0,
            rbx:           0,
        })
    }
}
