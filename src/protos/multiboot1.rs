#![allow(unaligned_references)]

//! Implementação do Protocolo de Boot Multiboot 1
//!
//! Implementa a especificação Multiboot 1 para carregamento de kernels.
//! Referência: https://www.gnu.org/software/grub/manual/multiboot/multiboot.html

use core::mem::size_of;

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

// Constantes Multiboot 1
const MB1_MAGIC: u32 = 0x1BADB002;
const MB1_BOOTLOADER_MAGIC: u32 = 0x2BADB002;
const MB1_HEADER_SEARCH: usize = 8192; // Procurar primeiros 8KB

// Flags Multiboot 1
const MB1_PAGE_ALIGN: u32 = 1 << 0;
const MB1_MEMORY_INFO: u32 = 1 << 1;
const MB1_VIDEO_MODE: u32 = 1 << 2;
const MB1_AOUT_KLUDGE: u32 = 1 << 16;

/// Cabeçalho Multiboot 1
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Multiboot1Header {
    magic:         u32,
    flags:         u32,
    checksum:      u32,
    // Presente apenas se MB1_AOUT_KLUDGE estiver definido
    header_addr:   u32,
    load_addr:     u32,
    load_end_addr: u32,
    bss_end_addr:  u32,
    entry_addr:    u32,
    // Presente apenas se MB1_VIDEO_MODE estiver definido
    mode_type:     u32,
    width:         u32,
    height:        u32,
    depth:         u32,
}

/// Estrutura de informação Multiboot 1 passada para o kernel
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Multiboot1Info {
    flags:              u32,
    mem_lower:          u32,
    mem_upper:          u32,
    boot_device:        u32,
    cmdline:            u32,
    mods_count:         u32,
    mods_addr:          u32,
    syms:               [u32; 4],
    mmap_length:        u32,
    mmap_addr:          u32,
    drives_length:      u32,
    drives_addr:        u32,
    config_table:       u32,
    boot_loader_name:   u32,
    apm_table:          u32,
    vbe_control_info:   u32,
    vbe_mode_info:      u32,
    vbe_mode:           u16,
    vbe_interface_seg:  u16,
    vbe_interface_off:  u16,
    vbe_interface_len:  u16,
    framebuffer_addr:   u64,
    framebuffer_pitch:  u32,
    framebuffer_width:  u32,
    framebuffer_height: u32,
    framebuffer_bpp:    u8,
    framebuffer_type:   u8,
    color_info:         [u8; 6],
}

/// Estrutura de módulo Multiboot 1
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Multiboot1Module {
    mod_start: u32,
    mod_end:   u32,
    string:    u32,
    reserved:  u32,
}

/// Implementação do protocolo Multiboot 1
pub struct Multiboot1Protocol<'a> {
    allocator:   &'a MemoryAllocator<'a>,
    header:      Option<Multiboot1Header>,
    entry_point: u64,
    load_addr:   u64,
}

impl<'a> Multiboot1Protocol<'a> {
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self {
            allocator,
            header: None,
            entry_point: 0,
            load_addr: 0,
        }
    }

    /// Encontrar e analisar cabeçalho Multiboot 1
    fn find_header(&self, kernel: &[u8]) -> Result<(Multiboot1Header, usize)> {
        let search_len = MB1_HEADER_SEARCH.min(kernel.len());

        // Procurar por magic nos primeiros 8KB, alinhado em fronteira de 4 bytes
        for offset in (0..search_len - size_of::<Multiboot1Header>()).step_by(4) {
            let magic = u32::from_le_bytes([
                kernel[offset],
                kernel[offset + 1],
                kernel[offset + 2],
                kernel[offset + 3],
            ]);

            if magic == MB1_MAGIC {
                // Magic encontrado, ler cabeçalho completo
                let header_bytes = &kernel[offset..offset + size_of::<Multiboot1Header>()];
                let header: Multiboot1Header = unsafe {
                    core::ptr::read_unaligned(header_bytes.as_ptr() as *const Multiboot1Header)
                };

                // Validar checksum
                let checksum = header
                    .magic
                    .wrapping_add(header.flags)
                    .wrapping_add(header.checksum);
                if checksum != 0 {
                    continue; // Checksum inválido, continuar procurando
                }

                info!("Multiboot 1 header found at offset {:#x}", offset);
                let flags = unsafe { core::ptr::read_unaligned(&raw const header.flags) };
                info!("  flags: {:#x}", flags);
                return Ok((header, offset));
            }
        }

        Err(BootError::Generic("Multiboot 1 header not found"))
    }

    /// Carregar kernel usando a.out kludge ou ELF
    fn load_kernel(&mut self, kernel: &[u8], header: &Multiboot1Header) -> Result<u64> {
        if (header.flags & MB1_AOUT_KLUDGE) != 0 {
            // a.out kludge - carregar em endereços específicos
            info!("Multiboot 1: Usando a.out kludge");

            let load_addr = header.load_addr as u64;
            let load_end = header.load_end_addr as u64;
            let bss_end = header.bss_end_addr as u64;
            let entry = header.entry_addr as u64;

            let load_size = (load_end - load_addr) as usize;

            info!("  load_addr: {:#x}", load_addr);
            info!("  load_end: {:#x}", load_end);
            info!("  bss_end: {:#x}", bss_end);
            info!("  entry: {:#x}", entry);

            // Alocar memória
            let pages = ((bss_end - load_addr) as usize + 4095) / 4096;
            let mem = self.allocator.allocate_at_address(load_addr, pages)?;

            // Copiar seção carregável
            unsafe {
                core::ptr::copy_nonoverlapping(kernel.as_ptr(), mem as *mut u8, load_size);

                // Zerar BSS
                let bss_start = load_end;
                let bss_size = (bss_end - bss_start) as usize;
                core::ptr::write_bytes((mem + (bss_start - load_addr)) as *mut u8, 0, bss_size);
            }

            self.entry_point = entry;
            self.load_addr = load_addr;

            Ok(load_addr)
        } else {
            // Formato ELF - usar carregador ELF
            info!("Multiboot 1: Usando formato ELF");

            use crate::elf::ElfLoader;
            let elf_loader = ElfLoader::new(self.allocator);
            let loaded = elf_loader.load(kernel)?;

            self.entry_point = loaded.entry_point;
            self.load_addr = loaded.base_address;

            Ok(loaded.base_address)
        }
    }

    /// Criar estrutura de informação Multiboot 1
    fn create_mbi(&self, cmdline: Option<&str>, modules: &[LoadedFile]) -> Result<u64> {
        // Alocar memória para MBI
        let mbi_ptr = self.allocator.allocate_any(1)?;
        let mbi = unsafe { &mut *(mbi_ptr as *mut Multiboot1Info) };

        // Inicializar MBI
        *mbi = Multiboot1Info {
            flags:              0,
            mem_lower:          0,
            mem_upper:          0,
            boot_device:        0,
            cmdline:            0,
            mods_count:         0,
            mods_addr:          0,
            syms:               [0; 4],
            mmap_length:        0,
            mmap_addr:          0,
            drives_length:      0,
            drives_addr:        0,
            config_table:       0,
            boot_loader_name:   0,
            apm_table:          0,
            vbe_control_info:   0,
            vbe_mode_info:      0,
            vbe_mode:           0,
            vbe_interface_seg:  0,
            vbe_interface_off:  0,
            vbe_interface_len:  0,
            framebuffer_addr:   0,
            framebuffer_pitch:  0,
            framebuffer_width:  0,
            framebuffer_height: 0,
            framebuffer_bpp:    0,
            framebuffer_type:   0,
            color_info:         [0; 6],
        };

        // TODO: Preencher:
        // - Info de memória (flags |= 1)
        // - Linha de comando (flags |= 4)
        // - Módulos (flags |= 8)
        // - Mapa de memória (flags |= 64)
        // - Info de Framebuffer (flags |= 4096)

        if let Some(cmd) = cmdline {
            // Alocar e copiar linha de comando
            let cmd_bytes = cmd.as_bytes();
            let cmd_ptr = self.allocator.allocate_any(1)?;
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmd_bytes.as_ptr(),
                    cmd_ptr as *mut u8,
                    cmd_bytes.len(),
                );
                *((cmd_ptr + cmd_bytes.len() as u64) as *mut u8) = 0;
            }
            mbi.cmdline = cmd_ptr as u32;
            mbi.flags |= 4;
        }

        info!("Multiboot 1 info at {:#x}", mbi_ptr);

        Ok(mbi_ptr)
    }
}

impl<'a> BootProtocol for Multiboot1Protocol<'a> {
    fn validate(&self, kernel: &[u8]) -> Result<()> {
        self.find_header(kernel)?;
        Ok(())
    }

    fn prepare(
        &mut self,
        kernel: &[u8],
        cmdline: Option<&str>,
        modules: &[LoadedFile],
    ) -> Result<BootInfo> {
        // Encontrar e analisar cabeçalho
        let (header, _offset) = self.find_header(kernel)?;
        self.header = Some(header);

        // Carregar kernel
        self.load_kernel(kernel, &header)?;

        // Criar MBI
        let mbi_ptr = self.create_mbi(cmdline, modules)?;

        Ok(BootInfo {
            entry_point:   self.entry_point,
            kernel_base:   self.load_addr,
            kernel_size:   kernel.len() as u64,
            stack_ptr:     None,
            boot_info_ptr: mbi_ptr,
            registers:     ProtocolRegisters {
                rax: Some(MB1_BOOTLOADER_MAGIC as u64), // EAX = magic
                rbx: Some(mbi_ptr),                     // EBX = ponteiro MBI
                ..Default::default()
            },
        })
    }

    fn entry_point(&self) -> u64 {
        self.entry_point
    }

    fn name(&self) -> &'static str {
        "multiboot1"
    }
}
