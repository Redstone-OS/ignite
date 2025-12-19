#![allow(unaligned_references)]

//! Linux Boot Protocol Implementation
//!
//! Implements the Linux x86/x86_64 boot protocol for loading Linux kernels.
//! Reference: https://www.kernel.org/doc/html/latest/x86/boot.html

use core::mem::size_of;

use log::info;

use super::{BootInfo, BootProtocol, ProtocolRegisters};
use crate::{
    error::{BootError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

// Linux boot protocol constants
const LINUX_MAGIC: u32 = 0x53726448; // "HdrS"
const BOOT_FLAG_MAGIC: u16 = 0xAA55;
const SETUP_HEADER_OFFSET: usize = 0x1F1;

/// Linux kernel setup header (partial)
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
    header:                u32, // "HdrS" magic
    version:               u16, // Boot protocol version
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

/// Linux boot protocol implementation
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

    /// Parse Linux setup header from kernel image
    fn parse_setup_header(&self, kernel: &[u8]) -> Result<SetupHeader> {
        if kernel.len() < SETUP_HEADER_OFFSET + size_of::<SetupHeader>() {
            return Err(BootError::Generic(
                "Kernel too small for Linux boot protocol",
            ));
        }

        // Read setup header
        let header_bytes =
            &kernel[SETUP_HEADER_OFFSET..SETUP_HEADER_OFFSET + size_of::<SetupHeader>()];
        let header: SetupHeader =
            unsafe { core::ptr::read_unaligned(header_bytes.as_ptr() as *const SetupHeader) };

        // Validate magic numbers
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

    /// Load kernel into memory at appropriate address
    fn load_kernel(&mut self, kernel: &[u8], header: &SetupHeader) -> Result<u64> {
        // Calculate kernel load address
        let kernel_addr = if header.relocatable_kernel != 0 {
            // Relocatable kernel - can load at preferred address or elsewhere
            header.pref_address
        } else {
            // Non-relocatable - must load at specific address
            0x100000 // Default 1MB for non-relocatable kernels
        };

        // Setup sectors: first part of bzImage
        let setup_size = ((header.setup_sects as usize) + 1) * 512;

        // Protected mode kernel starts after setup
        let kernel_start = setup_size;
        let kernel_size = kernel.len() - kernel_start;

        info!(
            "Linux kernel: setup_size={:#x}, kernel_size={:#x}",
            setup_size, kernel_size
        );
        info!("Loading Linux kernel at {:#x}", kernel_addr);

        // Allocate memory for kernel
        let pages_needed = (kernel_size + 4095) / 4096;
        let kernel_mem = self
            .allocator
            .allocate_at_address(kernel_addr, pages_needed)?;

        // Copy kernel to memory
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

    /// Load initrd/initramfs if provided
    fn load_initrd(&mut self, modules: &[LoadedFile]) -> Result<()> {
        if modules.is_empty() {
            return Ok(());
        }

        // First module is initrd
        let initrd = &modules[0];

        info!(
            "Loading initrd at {:#x} ({} bytes)",
            initrd.ptr, initrd.size
        );
        self.initrd_addr = Some(initrd.ptr);

        Ok(())
    }

    /// Setup command line
    fn setup_cmdline(&mut self, cmdline: Option<&str>) -> Result<()> {
        if let Some(cmd) = cmdline {
            let cmdline_bytes = cmd.as_bytes();
            let size = cmdline_bytes.len() + 1; // +1 for null terminator

            // Allocate memory for command line
            let pages = (size + 4095) / 4096;
            let cmdline_ptr = self.allocator.allocate_any(pages)?;

            // Copy command line
            unsafe {
                core::ptr::copy_nonoverlapping(
                    cmdline_bytes.as_ptr(),
                    cmdline_ptr as *mut u8,
                    cmdline_bytes.len(),
                );
                // Null terminate
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
        // Parse setup header
        let header = self.parse_setup_header(kernel)?;
        self.setup_header = Some(header);

        // Load kernel
        let kernel_addr = self.load_kernel(kernel, &header)?;

        // Load initrd if provided
        self.load_initrd(modules)?;

        // Setup command line
        self.setup_cmdline(cmdline)?;

        // Create boot parameters structure
        // TODO: Allocate and fill Linux boot_params structure
        // This includes:
        // - Setup header
        // - Command line pointer
        // - Initrd address and size
        // - E820 memory map
        // - Framebuffer info
        // - etc.

        let boot_params_ptr = 0; // TODO: Allocate boot_params

        // Entry point is code32_start or pref_address
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
                rsi: Some(boot_params_ptr), // RSI = boot_params pointer
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
