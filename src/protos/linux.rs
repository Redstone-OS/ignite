#![allow(unaligned_references)]

//! Implementação do Protocolo de Boot Linux
//!
//! Implementa o protocolo de boot Linux x86/x86_64 para carregar kernels Linux.
//! Referência: https://www.kernel.org/doc/html/latest/x86/boot.html

use core::mem::size_of;

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

// Constantes do protocolo de boot Linux
const LINUX_MAGIC: u32 = 0x53726448; // "HdrS"
const BOOT_FLAG_MAGIC: u16 = 0xAA55;
const SETUP_HEADER_OFFSET: usize = 0x1F1;

/// Cabeçalho de setup do kernel Linux (parcial)
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct SetupHeader {
    setup_sects:           u8,
    root_flags:            u16,
    syssize:               u32,
    ram_size:              u16,
    vid_mode:              u16,
    root_dev:              u16,
    boot_flag:             u16,
    jump:                  u16,
    header:                u32, // Magic "HdrS"
    version:               u16, // Versão do protocolo de boot
    realmode_swtch:        u32,
    start_sys_seg:         u16,
    kernel_version:        u16,
    type_of_loader:        u8,
    loadflags:             u8,
    setup_move_size:       u16,
    code32_start:          u32,
    ramdisk_image:         u32,
    ramdisk_size:          u32,
    bootsect_kludge:       u32,
    heap_end_ptr:          u16,
    ext_loader_ver:        u8,
    ext_loader_type:       u8,
    cmd_line_ptr:          u32,
    initrd_addr_max:       u32,
    kernel_alignment:      u32,
    relocatable_kernel:    u8,
    min_alignment:         u8,
    xloadflags:            u16,
    cmdline_size:          u32,
    hardware_subarch:      u32,
    hardware_subarch_data: u64,
    payload_offset:        u32,
    payload_length:        u32,
    setup_data:            u64,
    pref_address:          u64,
    init_size:             u32,
    handover_offset:       u32,
}

/// Implementação do protocolo de boot Linux
pub struct LinuxProtocol<'a> {
    allocator:    &'a MemoryAllocator<'a>,
    setup_header: Option<SetupHeader>,
    kernel_addr:  u64,
    initrd_addr:  Option<u64>,
    cmdline_addr: Option<u64>,
}

impl<'a> LinuxProtocol<'a> {
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self {
            allocator,
            setup_header: None,
            kernel_addr: 0,
            initrd_addr: None,
            cmdline_addr: None,
        }
    }

    /// Analisar cabeçalho de setup Linux da imagem do kernel
    fn parse_setup_header(&self, kernel: &[u8]) -> Result<SetupHeader> {
        if kernel.len() < SETUP_HEADER_OFFSET + size_of::<SetupHeader>() {
            return Err(BootError::Generic(
                "Kernel too small for Linux boot protocol",
            ));
        }

        // Ler cabeçalho de setup
        let header_bytes =
            &kernel[SETUP_HEADER_OFFSET..SETUP_HEADER_OFFSET + size_of::<SetupHeader>()];
        let header: SetupHeader =
            unsafe { core::ptr::read_unaligned(header_bytes.as_ptr() as *const SetupHeader) };

        // Validar números mágicos
        if header.boot_flag != BOOT_FLAG_MAGIC {
            return Err(BootError::Generic("Invalid Linux boot flag"));
        }

        if header.header != LINUX_MAGIC {
            return Err(BootError::Generic(
                "Invalid Linux header magic (not a bzImage)",
            ));
        }

        let version = unsafe { core::ptr::read_unaligned(&raw const header.version) };
        info!("Linux boot protocol version: {:#x}", version);
        info!(
            "Linux kernel {} relocatable",
            if header.relocatable_kernel != 0 {
                "is"
            } else {
                "is not"
            }
        );

        Ok(header)
    }

    /// Carregar kernel na memória em endereço apropriado
    fn load_kernel(&mut self, kernel: &[u8], header: &SetupHeader) -> Result<u64> {
        // Calcular endereço de carregamento do kernel
        let kernel_addr = if header.relocatable_kernel != 0 {
            // Kernel relocável - pode carregar no endereço preferido ou em outro lugar
            header.pref_address
        } else {
            // Não-relocável - deve carregar em endereço específico
            0x100000 // Padrão 1MB para kernels não-relocáveis
        };

        // Setores de setup: primeira parte do bzImage
        let setup_size = ((header.setup_sects as usize) + 1) * 512;

        // Kernel em modo protegido começa após setup
        let kernel_start = setup_size;
        let kernel_size = kernel.len() - kernel_start;

        info!(
            "Linux kernel: setup_size={:#x}, kernel_size={:#x}",
            setup_size, kernel_size
        );
        info!("Loading Linux kernel at {:#x}", kernel_addr);

        // Alocar memória para kernel
        let pages_needed = (kernel_size + 4095) / 4096;
        let kernel_mem = self
            .allocator
            .allocate_at_address(kernel_addr, pages_needed)?;

        // Copiar kernel para memória
        unsafe {
            core::ptr::copy_nonoverlapping(
                kernel[kernel_start..].as_ptr(),
                kernel_mem as *mut u8,
                kernel_size,
            );
        }

        self.kernel_addr = kernel_addr;
        Ok(kernel_addr)
    }

    /// Carregar initrd/initramfs se fornecido
    fn load_initrd(&mut self, modules: &[LoadedFile]) -> Result<()> {
        if modules.is_empty() {
            return Ok(());
        }

        // Primeiro módulo é initrd
        let initrd = &modules[0];

        info!(
            "Loading initrd at {:#x} ({} bytes)",
            initrd.ptr, initrd.size
        );
        self.initrd_addr = Some(initrd.ptr);

        Ok(())
    }

    /// Configurar linha de comando
    fn setup_cmdline(&mut self, cmdline: Option<&str>) -> Result<()> {
        if let Some(cmd) = cmdline {
            let cmdline_bytes = cmd.as_bytes();
            let size = cmdline_bytes.len() + 1; // +1 for null terminator

            // Alocar memória para linha de comando
            let pages = (size + 4095) / 4096;
            let cmdline_ptr = self.allocator.allocate_any(pages)?;

            // Copiar linha de comando
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmdline_bytes.as_ptr(),
                    cmdline_ptr as *mut u8,
                    cmdline_bytes.len(),
                );
                // Terminar com nulo
                *((cmdline_ptr + cmdline_bytes.len() as u64) as *mut u8) = 0;
            }

            self.cmdline_addr = Some(cmdline_ptr);
            info!("Linux cmdline at {:#x}: {}", cmdline_ptr, cmd);
        }

        Ok(())
    }
}

impl<'a> BootProtocol for LinuxProtocol<'a> {
    fn validate(&self, kernel: &[u8]) -> Result<()> {
        self.parse_setup_header(kernel)?;
        Ok(())
    }

    fn prepare(
        &mut self,
        kernel: &[u8],
        cmdline: Option<&str>,
        modules: &[LoadedFile],
    ) -> Result<BootInfo> {
        // Analisar cabeçalho de setup
        let header = self.parse_setup_header(kernel)?;
        self.setup_header = Some(header);

        // Carregar kernel
        let kernel_addr = self.load_kernel(kernel, &header)?;

        // Carregar initrd se fornecido
        self.load_initrd(modules)?;

        // Configurar linha de comando
        self.setup_cmdline(cmdline)?;

        // Criar estrutura de parâmetros de boot
        // TODO: Alocar e preencher estrutura boot_params do Linux
        // Isso inclui:
        // - Cabeçalho de setup
        // - Ponteiro da linha de comando
        // - Endereço e tamanho do initrd
        // - Mapa de memória E820
        // - Info de Framebuffer
        // - etc.

        let boot_params_ptr = 0; // TODO: Alocar boot_params

        // Ponto de entrada é code32_start ou pref_address
        let entry_point = if header.code32_start != 0 {
            header.code32_start as u64
        } else {
            kernel_addr
        };

        info!("Linux entry point: {:#x}", entry_point);

        Ok(BootInfo {
            entry_point,
            kernel_base: kernel_addr,
            kernel_size: kernel.len() as u64,
            stack_ptr: None,
            boot_info_ptr: boot_params_ptr,
            registers: ProtocolRegisters {
                rsi: Some(boot_params_ptr), // RSI = ponteiro boot_params
                ..Default::default()
            },
        })
    }

    fn entry_point(&self) -> u64 {
        self.kernel_addr
    }

    fn name(&self) -> &'static str {
        "linux"
    }
}
