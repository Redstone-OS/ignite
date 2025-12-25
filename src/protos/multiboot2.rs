//! Protocolo Multiboot 2
//!
//! Suporte para kernels compatíveis com GRUB (Multiboot 2).

use alloc::vec::Vec;

use super::{BootProtocol, KernelLaunchInfo};
use crate::{
    core::{
        error::{BootError, Result},
        types::LoadedFile,
    },
    memory::FrameAllocator,
};

const MB2_MAGIC: u32 = 0xE85250D6;

pub struct Multiboot2Protocol<'a> {
    allocator: &'a mut dyn FrameAllocator,
}

impl<'a> Multiboot2Protocol<'a> {
    pub fn new(allocator: &'a mut dyn FrameAllocator) -> Self {
        Self { allocator }
    }
}

impl<'a> BootProtocol for Multiboot2Protocol<'a> {
    fn name(&self) -> &str {
        "Multiboot 2"
    }

    fn identify(&self, file_content: &[u8]) -> bool {
        // Procurar magic nos primeiros 32KB
        let _search_limit = core::cmp::min(file_content.len(), 32768);
        // Implementar busca alinhada a 8 bytes
        false // Placeholder
    }

    fn load(
        &mut self,
        _kernel_file: &[u8],
        _cmdline: Option<&str>,
        _modules: Vec<LoadedFile>,
        _memory_map_buffer: (u64, u64),
        _framebuffer: Option<crate::core::handoff::FramebufferInfo>,
    ) -> Result<KernelLaunchInfo> {
        Err(BootError::Generic("Multiboot2 ainda não implementado"))
    }
}
