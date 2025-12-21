//! EFI Boot Services Table
//!
//! Referência: UEFI Spec 2.10, Seção 7 - Services - Boot Services

use super::system::TableHeader;
use crate::uefi::base::*;

/// Memory Type
///
/// Spec: 7.2 - Memory Allocation Services
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MemoryType {
    ReservedMemoryType = 0,
    LoaderCode = 1,
    LoaderData = 2,
    BootServicesCode = 3,
    BootServicesData = 4,
    RuntimeServicesCode = 5,
    RuntimeServicesData = 6,
    ConventionalMemory = 7,
    UnusableMemory = 8,
    ACPIReclaimMemory = 9,
    ACPIMemoryNVS = 10,
    MemoryMappedIO = 11,
    MemoryMappedIOPortSpace = 12,
    PalCode = 13,
    PersistentMemory = 14,
    UnacceptedMemoryType = 15,
    MaxMemoryType = 16,
}

// Aliases para compatibilidade com código existente
impl MemoryType {
    pub const CONVENTIONAL: Self = Self::ConventionalMemory;
    pub const ACPI_RECLAIM: Self = Self::ACPIReclaimMemory;
    pub const ACPI_NON_VOLATILE: Self = Self::ACPIMemoryNVS;
}

/// Allocate Type
///
/// Spec: 7.2 - Memory Allocation Services
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum AllocateType {
    /// Alocar qualquer página disponível
    AllocateAnyPages = 0,
    /// Alocar no endereço máximo especificado
    AllocateMaxAddress = 1,
    /// Alocar no endereço específico
    AllocateAddress = 2,
    MaxAllocateType = 3,
}

/// Memory Descriptor
///
/// Spec: 7.2 - EFI_MEMORY_DESCRIPTOR
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MemoryDescriptor {
    pub ty:              u32,
    pub physical_start:  u64,
    pub virtual_start:   u64,
    pub number_of_pages: u64,
    pub attribute:       u64,
}

// Memory Attributes - Spec: 7.2
pub const MEMORY_UC: u64 = 0x0000000000000001;
pub const MEMORY_WC: u64 = 0x0000000000000002;
pub const MEMORY_WT: u64 = 0x0000000000000004;
pub const MEMORY_WB: u64 = 0x0000000000000008;
pub const MEMORY_UCE: u64 = 0x0000000000000010;
pub const MEMORY_WP: u64 = 0x0000000000001000;
pub const MEMORY_RP: u64 = 0x0000000000002000;
pub const MEMORY_XP: u64 = 0x0000000000004000;
pub const MEMORY_NV: u64 = 0x0000000000008000;
pub const MEMORY_MORE_RELIABLE: u64 = 0x0000000000010000;
pub const MEMORY_RO: u64 = 0x0000000000020000;
pub const MEMORY_SP: u64 = 0x0000000000040000;
pub const MEMORY_CPU_CRYPTO: u64 = 0x0000000000080000;
pub const MEMORY_RUNTIME: u64 = 0x8000000000000000;

/// Task Priority Level (TPL)
///
/// Spec: 7.1 - Event, Timer, and Task Priority Services
pub type Tpl = usize;

pub const TPL_APPLICATION: Tpl = 4;
pub const TPL_CALLBACK: Tpl = 8;
pub const TPL_NOTIFY: Tpl = 16;
pub const TPL_HIGH_LEVEL: Tpl = 31;

/// Interface Type
#[repr(u32)]
pub enum InterfaceType {
    NativeInterface = 0,
}

/// Locate Search Type
#[repr(u32)]
pub enum LocateSearchType {
    AllHandles = 0,
    ByRegisterNotify = 1,
    ByProtocol = 2,
}

/// Open Protocol Attributes
pub const OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 0x00000001;
pub const OPEN_PROTOCOL_GET_PROTOCOL: u32 = 0x00000002;
pub const OPEN_PROTOCOL_TEST_PROTOCOL: u32 = 0x00000004;
pub const OPEN_PROTOCOL_BY_CHILD_CONTROLLER: u32 = 0x00000008;
pub const OPEN_PROTOCOL_BY_DRIVER: u32 = 0x00000010;
pub const OPEN_PROTOCOL_EXCLUSIVE: u32 = 0x00000020;

/// EFI Boot Services Table
///
/// Spec: 4.4 - EFI Boot Services Table
#[repr(C)]
pub struct BootServices {
    /// EFI Table Header
    pub hdr: TableHeader,

    // ===== Task Priority Services (7.1) =====
    pub raise_tpl:   extern "efiapi" fn(Tpl) -> Tpl,
    pub restore_tpl: extern "efiapi" fn(Tpl),

    // ===== Memory Services (7.2) =====
    pub allocate_pages: extern "efiapi" fn(
        AllocateType,
        MemoryType,
        usize,    // Pages
        *mut u64, // Memory (in/out)
    ) -> Status,

    pub free_pages: extern "efiapi" fn(
        u64,   // Memory
        usize, // Pages
    ) -> Status,

    pub get_memory_map: extern "efiapi" fn(
        *mut usize,            // MemoryMapSize (in/out)
        *mut MemoryDescriptor, // MemoryMap
        *mut usize,            // MapKey
        *mut usize,            // DescriptorSize
        *mut u32,              // DescriptorVersion
    ) -> Status,

    pub allocate_pool: extern "efiapi" fn(
        MemoryType,
        usize,        // Size
        *mut *mut u8, // Buffer
    ) -> Status,

    pub free_pool: extern "efiapi" fn(*mut u8) -> Status,

    // ===== Event & Timer Services (7.1) =====
    pub create_event: extern "efiapi" fn(
        u32,                    // Type
        Tpl,                    // NotifyTpl
        usize,                  // NotifyFunction
        *mut core::ffi::c_void, // NotifyContext
        *mut Event,             // Event
    ) -> Status,

    pub set_timer: extern "efiapi" fn(
        Event,
        u32, // Type
        u64, // TriggerTime
    ) -> Status,

    pub wait_for_event: extern "efiapi" fn(
        usize,      // NumberOfEvents
        *mut Event, // Event
        *mut usize, // Index
    ) -> Status,

    pub signal_event: extern "efiapi" fn(Event) -> Status,
    pub close_event:  extern "efiapi" fn(Event) -> Status,
    pub check_event:  extern "efiapi" fn(Event) -> Status,

    // ===== Protocol Handler Services (7.3) =====
    pub install_protocol_interface: extern "efiapi" fn(
        *mut Handle,
        *const Guid,
        InterfaceType,
        *mut core::ffi::c_void,
    ) -> Status,

    pub reinstall_protocol_interface: extern "efiapi" fn(
        Handle,
        *const Guid,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> Status,

    pub uninstall_protocol_interface:
        extern "efiapi" fn(Handle, *const Guid, *mut core::ffi::c_void) -> Status,

    pub handle_protocol:
        extern "efiapi" fn(Handle, *const Guid, *mut *mut core::ffi::c_void) -> Status,

    pub reserved: *mut core::ffi::c_void,

    pub register_protocol_notify:
        extern "efiapi" fn(*const Guid, Event, *mut *mut core::ffi::c_void) -> Status,

    pub locate_handle: extern "efiapi" fn(
        LocateSearchType,
        *const Guid,
        *mut core::ffi::c_void,
        *mut usize,
        *mut Handle,
    ) -> Status,

    pub locate_device_path:
        extern "efiapi" fn(*const Guid, *mut *mut core::ffi::c_void, *mut Handle) -> Status,

    pub install_configuration_table:
        extern "efiapi" fn(*const Guid, *mut core::ffi::c_void) -> Status,

    // ===== Image Services (7.4) =====
    pub load_image: extern "efiapi" fn(
        Boolean,                // BootPolicy
        Handle,                 // ParentImageHandle
        *mut core::ffi::c_void, // DevicePath
        *mut core::ffi::c_void, // SourceBuffer
        usize,                  // SourceSize
        *mut Handle,            // ImageHandle
    ) -> Status,

    pub start_image: extern "efiapi" fn(Handle, *mut usize, *mut *mut Char16) -> Status,

    pub exit: extern "efiapi" fn(Handle, Status, usize, *mut Char16) -> Status,

    pub unload_image: extern "efiapi" fn(Handle) -> Status,

    pub exit_boot_services: extern "efiapi" fn(
        Handle, // ImageHandle
        usize,  // MapKey
    ) -> Status,

    // ===== Miscellaneous Services (7.5) =====
    pub get_next_monotonic_count: extern "efiapi" fn(*mut u64) -> Status,
    pub stall:                    extern "efiapi" fn(usize) -> Status,
    pub set_watchdog_timer:       extern "efiapi" fn(usize, u64, usize, *const Char16) -> Status,

    // ===== Driver Support Services (7.3) =====
    pub connect_controller:
        extern "efiapi" fn(Handle, *mut Handle, *mut core::ffi::c_void, Boolean) -> Status,

    pub disconnect_controller: extern "efiapi" fn(Handle, Handle, Handle) -> Status,

    // ===== Open and Close Protocol Services (7.3) =====
    pub open_protocol: extern "efiapi" fn(
        Handle,                      // Handle
        *const Guid,                 // Protocol
        *mut *mut core::ffi::c_void, // Interface
        Handle,                      // AgentHandle
        Handle,                      // ControllerHandle
        u32,                         // Attributes
    ) -> Status,

    pub close_protocol: extern "efiapi" fn(Handle, *const Guid, Handle, Handle) -> Status,

    pub open_protocol_information:
        extern "efiapi" fn(Handle, *const Guid, *mut *mut core::ffi::c_void, *mut usize) -> Status,

    // ===== Library Services (7.3) =====
    pub protocols_per_handle: extern "efiapi" fn(Handle, *mut *mut *mut Guid, *mut usize) -> Status,

    pub locate_handle_buffer: extern "efiapi" fn(
        LocateSearchType,
        *const Guid,
        *mut core::ffi::c_void,
        *mut usize,
        *mut *mut Handle,
    ) -> Status,

    pub locate_protocol: extern "efiapi" fn(
        *const Guid,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> Status,

    pub install_multiple_protocol_interfaces:   usize,
    pub uninstall_multiple_protocol_interfaces: usize,

    // ===== 32-bit CRC Services (7.5) =====
    pub calculate_crc32: extern "efiapi" fn(*mut core::ffi::c_void, usize, *mut u32) -> Status,

    // ===== Miscellaneous Services (7.5) =====
    pub copy_mem: extern "efiapi" fn(*mut core::ffi::c_void, *const core::ffi::c_void, usize),

    pub set_mem: extern "efiapi" fn(*mut core::ffi::c_void, usize, u8),

    pub create_event_ex: extern "efiapi" fn(
        u32,
        Tpl,
        usize,
        *const core::ffi::c_void,
        *const Guid,
        *mut Event,
    ) -> Status,
}

/// Event Types - Spec: 7.1
pub const EVT_TIMER: u32 = 0x80000000;
pub const EVT_RUNTIME: u32 = 0x40000000;
pub const EVT_NOTIFY_WAIT: u32 = 0x00000100;
pub const EVT_NOTIFY_SIGNAL: u32 = 0x00000200;
pub const EVT_SIGNAL_EXIT_BOOT_SERVICES: u32 = 0x00000201;
pub const EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE: u32 = 0x60000202;

/// Timer Delay - Spec: 7.1
#[repr(u32)]
pub enum TimerDelay {
    TimerCancel = 0,
    TimerPeriodic = 1,
    TimerRelative = 2,
}
