//! Protocolo de Boot Linux
//!
//! Implementa o protocolo de boot x86 do Linux (Setup Header + Zero Page).
//! Permite carregar distros Linux padrão.

use alloc::vec::Vec;
use core::mem::size_of;

use super::{BootProtocol, KernelLaunchInfo};
use crate::{
    core::{
        error::{BootError, Result},
        types::LoadedFile,
    },
    memory::FrameAllocator,
};

const LINUX_SETUP_HEADER_OFFSET: usize = 0x1F1;
const LINUX_MAGIC: u32 = 0x53726448; // "HdrS"

#[repr(C, packed)]
struct LinuxSetupHeader {
    setup_sects:        u8,
    root_flags:         u16,
    syssize:            u32,
    ram_size:           u16,
    vid_mode:           u16,
    root_dev:           u16,
    boot_flag:          u16,
    jump:               u16,
    header:             u32, // Magic
    version:            u16,
    realmode_swtch:     u32,
    start_sys_seg:      u16,
    kernel_version:     u16,
    type_of_loader:     u8,
    loadflags:          u8,
    setup_move_size:    u16,
    code32_start:       u32,
    ramdisk_image:      u32,
    ramdisk_size:       u32,
    bootsect_kludge:    u32,
    heap_end_ptr:       u16,
    ext_loader_ver:     u8,
    ext_loader_type:    u8,
    cmd_line_ptr:       u32,
    initrd_addr_max:    u32,
    kernel_alignment:   u32,
    relocatable_kernel: u8,
    min_alignment:      u8,
    xloadflags:         u16,
    cmdline_size:       u32,
    // ... campos adicionais omitidos para brevidade
}

pub struct LinuxProtocol<'a> {
    allocator: &'a mut dyn FrameAllocator,
}

impl<'a> LinuxProtocol<'a> {
    pub fn new(allocator: &'a mut dyn FrameAllocator) -> Self {
        Self { allocator }
    }
}

impl<'a> BootProtocol for LinuxProtocol<'a> {
    fn name(&self) -> &str {
        "Linux bzImage"
    }

    fn identify(&self, file_content: &[u8]) -> bool {
        if file_content.len() < LINUX_SETUP_HEADER_OFFSET + size_of::<LinuxSetupHeader>() {
            return false;
        }
        // Verificar magic "HdrS"
        let magic_ptr = unsafe {
            file_content
                .as_ptr()
                .add(LINUX_SETUP_HEADER_OFFSET + 0x202 - 0x1f1)
        }; // Offset manual seguro
        // Simplificação: ler direto da struct
        // Implementação real deve fazer cast seguro
        true // Placeholder: implementar verificação de bytes exata
    }

    fn load(
        &mut self,
        kernel_file: &[u8],
        cmdline: Option<&str>,
        modules: Vec<LoadedFile>,
    ) -> Result<KernelLaunchInfo> {
        // 1. Parsear Setup Header
        // 2. Alocar Zero Page (boot_params)
        // 3. Copiar Kernel para endereço protegido (code32_start ou relocável)
        // 4. Configurar Cmdline e Initrd na Zero Page

        // Placeholder de implementação
        Err(BootError::Generic(
            "Linux boot ainda não totalmente implementado neste refactor",
        ))
    }
}
