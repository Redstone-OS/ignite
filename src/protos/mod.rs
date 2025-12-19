//! Boot Protocol Abstraction
//!
//! This module defines traits and types for supporting multiple boot protocols.
//! Currently supported protocols:
//! - Limine Protocol (native)
//! - Linux Boot Protocol
//! - Multiboot 1
//! - Multiboot 2
//! - EFI/BIOS Chainloading

use alloc::vec::Vec;

use crate::{error::Result, types::LoadedFile};

pub mod chainload;
pub mod limine;
pub mod linux;
pub mod multiboot1;
pub mod multiboot2;

/// Boot protocol trait
///
/// All boot protocols must implement this trait to integrate with the
/// bootloader.
pub trait BootProtocol {
    /// Validate that the kernel/executable is compatible with this protocol
    fn validate(&self, kernel: &[u8]) -> Result<()>;

    /// Prepare boot information and kernel for handoff
    ///
    /// This method should:
    /// - Parse kernel headers
    /// - Load kernel segments into memory
    /// - Prepare protocol-specific boot information
    /// - Setup any required data structures
    fn prepare(
        &mut self,
        kernel: &[u8],
        cmdline: Option<&str>,
        modules: &[LoadedFile],
    ) -> Result<BootInfo>;

    /// Get the entry point address
    fn entry_point(&self) -> u64;

    /// Protocol name for logging
    fn name(&self) -> &'static str;
}

/// Boot information prepared by a protocol
#[derive(Debug)]
pub struct BootInfo {
    /// Entry point address to jump to
    pub entry_point: u64,

    /// Kernel base address in memory
    pub kernel_base: u64,

    /// Kernel size in bytes
    pub kernel_size: u64,

    /// Stack pointer (if protocol manages stack)
    pub stack_ptr: Option<u64>,

    /// Protocol-specific boot information pointer
    /// This will be passed to the kernel in RDI (x86_64 calling convention)
    pub boot_info_ptr: u64,

    /// Additional registers to set (protocol-specific)
    pub registers: ProtocolRegisters,
}

/// Protocol-specific register values
#[derive(Debug, Default)]
pub struct ProtocolRegisters {
    pub rax: Option<u64>,
    pub rbx: Option<u64>,
    pub rcx: Option<u64>,
    pub rdx: Option<u64>,
    pub rsi: Option<u64>,
    pub r8:  Option<u64>,
    pub r9:  Option<u64>,
}

/// Protocol type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// Native Limine protocol
    Limine,
    /// Linux boot protocol
    Linux,
    /// Multiboot version 1
    Multiboot1,
    /// Multiboot version 2
    Multiboot2,
    /// EFI chainloading
    EfiChainload,
    /// BIOS chainloading
    BiosChainload,
}

impl ProtocolType {
    /// Parse protocol type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "limine" | "native" => Some(Self::Limine),
            "linux" => Some(Self::Linux),
            "multiboot" | "multiboot1" => Some(Self::Multiboot1),
            "multiboot2" => Some(Self::Multiboot2),
            "efi" | "uefi" | "efi_chainload" => Some(Self::EfiChainload),
            "bios" | "bios_chainload" => Some(Self::BiosChainload),
            _ => None,
        }
    }

    /// Get protocol name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Limine => "limine",
            Self::Linux => "linux",
            Self::Multiboot1 => "multiboot1",
            Self::Multiboot2 => "multiboot2",
            Self::EfiChainload => "efi_chainload",
            Self::BiosChainload => "bios_chainload",
        }
    }
}
