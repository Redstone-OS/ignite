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
    setup_sects: u8,
    root_flags:  u16,
    syssize:     u32,
    ram_size:    u16,
    vid_mode:    u16,
    root_dev:    u16,
    boot_flag:   u16,
    jump:        u16,
    header:      u32, /* Magic "HdrS"
                       * ... restante dos campos omitidos para brevidade do check ... */
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
        // Verifica tamanho mínimo
        if file_content.len() < 0x202 + 4 {
            return false;
        }

        // Verifica assinatura "HdrS" no offset 0x202
        // O offset 0x202 é fixo no protocolo de boot Linux x86
        let magic_slice = &file_content[0x202..0x206];
        let magic = u32::from_le_bytes(magic_slice.try_into().unwrap_or([0; 4]));

        magic == LINUX_MAGIC
    }

    fn load(
        &mut self,
        _kernel_file: &[u8],
        _cmdline: Option<&str>,
        _modules: Vec<LoadedFile>,
        _memory_map_buffer: (u64, u64),
        _framebuffer: Option<crate::core::handoff::FramebufferInfo>,
    ) -> Result<KernelLaunchInfo> {
        // Implementação real seria aqui (parsing do setup header, alocação da zero
        // page, etc.)
        Err(BootError::Generic("Linux boot ainda não implementado"))
    }
}
