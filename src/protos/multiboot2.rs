//! Implementação do Protocolo de Boot Multiboot 2
//!
//! Implementa a especificação Multiboot 2.
//! Referência: https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

// Constantes Multiboot 2
const MB2_MAGIC: u32 = 0xE85250D6;
const MB2_BOOTLOADER_MAGIC: u32 = 0x36D76289;
const MB2_ARCHITECTURE_I386: u32 = 0;
const MB2_HEADER_TAG_END: u16 = 0;

/// Cabeçalho Multiboot 2
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Multiboot2Header {
    magic:         u32,
    architecture:  u32,
    header_length: u32,
    checksum:      u32,
}

/// Implementação do protocolo Multiboot 2
pub struct Multiboot2Protocol<'a> {
    allocator:   &'a MemoryAllocator<'a>,
    entry_point: u64,
    load_addr:   u64,
}

impl<'a> Multiboot2Protocol<'a> {
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self {
            allocator,
            entry_point: 0,
            load_addr: 0,
        }
    }

    /// Encontrar cabeçalho Multiboot 2
    fn find_header(&self, kernel: &[u8]) -> Result<(Multiboot2Header, usize)> {
        let search_len = 32768.min(kernel.len()); // Procurar primeiros 32KB

        for offset in (0..search_len - 16).step_by(8) {
            if offset + 16 > kernel.len() {
                break;
            }

            let magic = u32::from_le_bytes([
                kernel[offset],
                kernel[offset + 1],
                kernel[offset + 2],
                kernel[offset + 3],
            ]);

            if magic == MB2_MAGIC {
                let architecture = u32::from_le_bytes([
                    kernel[offset + 4],
                    kernel[offset + 5],
                    kernel[offset + 6],
                    kernel[offset + 7],
                ]);
                let header_length = u32::from_le_bytes([
                    kernel[offset + 8],
                    kernel[offset + 9],
                    kernel[offset + 10],
                    kernel[offset + 11],
                ]);
                let checksum = u32::from_le_bytes([
                    kernel[offset + 12],
                    kernel[offset + 13],
                    kernel[offset + 14],
                    kernel[offset + 15],
                ]);

                // Validar checksum
                let sum = magic
                    .wrapping_add(architecture)
                    .wrapping_add(header_length)
                    .wrapping_add(checksum);

                if sum == 0 {
                    let header = Multiboot2Header {
                        magic,
                        architecture,
                        header_length,
                        checksum,
                    };

                    info!("Multiboot 2 header found at offset {:#x}", offset);
                    info!("  architecture: {}", architecture);
                    info!("  header_length: {}", header_length);

                    return Ok((header, offset));
                }
            }
        }

        Err(BootError::Generic("Multiboot 2 header not found"))
    }
}

impl<'a> BootProtocol for Multiboot2Protocol<'a> {
    fn validate(&self, kernel: &[u8]) -> Result<()> {
        self.find_header(kernel)?;
        Ok(())
    }

    fn prepare(
        &mut self,
        kernel: &[u8],
        _cmdline: Option<&str>,
        _modules: &[LoadedFile],
    ) -> Result<BootInfo> {
        let (_header, _offset) = self.find_header(kernel)?;

        // Carregar como ELF (Multiboot 2 geralmente usa ELF)
        use crate::elf::ElfLoader;
        let elf_loader = ElfLoader::new(self.allocator);
        let loaded = elf_loader.load(kernel)?;

        self.entry_point = loaded.entry_point;
        self.load_addr = loaded.base_address;

        // TODO: Criar estrutura de informação Multiboot 2 com tags:
        // - Tag de linha de comando de boot
        // - Tag de nome do boot loader
        // - Tags de módulo
        // - Tag de mapa de memória
        // - Tag de info de framebuffer
        // - Tag de símbolos ELF
        // - etc.

        let mbi_ptr = 0; // TODO: Alocar e criar MBI

        Ok(BootInfo {
            entry_point:   self.entry_point,
            kernel_base:   self.load_addr,
            kernel_size:   kernel.len() as u64,
            stack_ptr:     None,
            boot_info_ptr: mbi_ptr,
            registers:     ProtocolRegisters {
                rax: Some(MB2_BOOTLOADER_MAGIC as u64), // EAX = magic
                rbx: Some(mbi_ptr),                     // EBX = ponteiro MBI
                ..Default::default()
            },
        })
    }

    fn entry_point(&self) -> u64 {
        self.entry_point
    }

    fn name(&self) -> &'static str {
        "multiboot2"
    }
}
