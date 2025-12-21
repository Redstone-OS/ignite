//! Boot Services - Serviços de Inicialização
//!
//! Contém as definições da tabela de serviços de boot e implementações seguras
//! para alocação de memória, manipulação de protocolos e eventos.

use core::ffi::c_void;

use crate::uefi::{
    Result,
    base::{Char16, Event, Guid, Handle, Status},
    table::header::TableHeader,
};

// --- Tipos de Enumeração ---

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum AllocateType {
    AllocateAnyPages = 0,
    AllocateMaxAddress = 1,
    AllocateAddress = 2,
    MaxAllocateType = 3,
}

#[repr(u32)]
pub enum InterfaceType {
    NativeInterface = 0,
}

#[repr(u32)]
pub enum LocateSearchType {
    AllHandles = 0,
    ByRegisterNotify = 1,
    ByProtocol = 2,
}

#[repr(u32)]
pub enum TimerDelay {
    TimerCancel = 0,
    TimerPeriodic = 1,
    TimerRelative = 2,
}

// Atributos para OpenProtocol
pub const OPEN_PROTOCOL_BY_HANDLE_PROTOCOL: u32 = 0x00000001;
pub const OPEN_PROTOCOL_GET_PROTOCOL: u32 = 0x00000002;
pub const OPEN_PROTOCOL_TEST_PROTOCOL: u32 = 0x00000004;
pub const OPEN_PROTOCOL_BY_CHILD_CONTROLLER: u32 = 0x00000008;
pub const OPEN_PROTOCOL_BY_DRIVER: u32 = 0x00000010;
pub const OPEN_PROTOCOL_EXCLUSIVE: u32 = 0x00000020;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MemoryDescriptor {
    pub ty:              u32,
    pub physical_start:  u64,
    pub virtual_start:   u64,
    pub number_of_pages: u64,
    pub attribute:       u64,
}

// --- Tabela BootServices (FFI) ---

#[repr(C)]
pub struct BootServices {
    pub hdr: TableHeader,

    // Task Priority Services (TPL)
    pub raise_tpl:   extern "efiapi" fn(usize) -> usize,
    pub restore_tpl: extern "efiapi" fn(usize),

    // Memory Services
    pub allocate_pages: extern "efiapi" fn(AllocateType, MemoryType, usize, *mut u64) -> Status,
    pub free_pages:     extern "efiapi" fn(u64, usize) -> Status,
    pub get_memory_map: extern "efiapi" fn(
        *mut usize,
        *mut MemoryDescriptor,
        *mut usize,
        *mut usize,
        *mut u32,
    ) -> Status,
    pub allocate_pool:  extern "efiapi" fn(MemoryType, usize, *mut *mut u8) -> Status,
    pub free_pool:      extern "efiapi" fn(*mut u8) -> Status,

    // Event & Timer Services
    pub create_event:   extern "efiapi" fn(u32, usize, usize, *mut c_void, *mut Event) -> Status,
    pub set_timer:      extern "efiapi" fn(Event, TimerDelay, u64) -> Status,
    pub wait_for_event: extern "efiapi" fn(usize, *mut Event, *mut usize) -> Status,
    pub signal_event:   extern "efiapi" fn(Event) -> Status,
    pub close_event:    extern "efiapi" fn(Event) -> Status,
    pub check_event:    extern "efiapi" fn(Event) -> Status,

    // Protocol Handler Services
    pub install_protocol_interface:
        extern "efiapi" fn(*mut Handle, *const Guid, InterfaceType, *mut c_void) -> Status,
    pub reinstall_protocol_interface:
        extern "efiapi" fn(Handle, *const Guid, *mut c_void, *mut c_void) -> Status,
    pub uninstall_protocol_interface:
        extern "efiapi" fn(Handle, *const Guid, *mut c_void) -> Status,
    pub handle_protocol: extern "efiapi" fn(Handle, *const Guid, *mut *mut c_void) -> Status,
    pub reserved:                     *mut c_void,
    pub register_protocol_notify:
        extern "efiapi" fn(*const Guid, Event, *mut *mut c_void) -> Status,
    pub locate_handle: extern "efiapi" fn(
        LocateSearchType,
        *const Guid,
        *mut c_void,
        *mut usize,
        *mut Handle,
    ) -> Status,
    pub locate_device_path:
        extern "efiapi" fn(*const Guid, *mut *mut c_void, *mut Handle) -> Status,
    pub install_configuration_table:  extern "efiapi" fn(*const Guid, *mut c_void) -> Status,

    // Image Services
    pub load_image:
        extern "efiapi" fn(u8, Handle, *mut c_void, *mut c_void, usize, *mut Handle) -> Status,
    pub start_image:        extern "efiapi" fn(Handle, *mut usize, *mut *mut u16) -> Status,
    pub exit:               extern "efiapi" fn(Handle, Status, usize, *mut u16) -> Status,
    pub unload_image:       extern "efiapi" fn(Handle) -> Status,
    pub exit_boot_services: extern "efiapi" fn(Handle, usize) -> Status,

    // Miscellaneous Services
    pub get_next_monotonic_count: extern "efiapi" fn(*mut u64) -> Status,
    pub stall:                    extern "efiapi" fn(usize) -> Status,
    pub set_watchdog_timer:       extern "efiapi" fn(usize, u64, usize, *const Char16) -> Status,

    // Driver Support Services
    pub connect_controller:    extern "efiapi" fn(Handle, *mut Handle, *mut c_void, u8) -> Status,
    pub disconnect_controller: extern "efiapi" fn(Handle, Handle, Handle) -> Status,

    // Open and Close Protocol Services
    pub open_protocol:
        extern "efiapi" fn(Handle, *const Guid, *mut *mut c_void, Handle, Handle, u32) -> Status,
    pub close_protocol: extern "efiapi" fn(Handle, *const Guid, Handle, Handle) -> Status,
    pub open_protocol_information:
        extern "efiapi" fn(Handle, *const Guid, *mut *mut c_void, *mut usize) -> Status,
}

// --- Métodos Seguros (Rust API) ---

impl BootServices {
    /// Aloca páginas de memória física.
    pub fn allocate_pages(
        &self,
        ty: AllocateType,
        memory_type: MemoryType,
        pages: usize,
    ) -> Result<u64> {
        let mut addr = 0;
        unsafe { (self.allocate_pages)(ty, memory_type, pages, &mut addr).to_result_with(addr) }
    }

    /// Aloca memória na heap do UEFI (Pool).
    pub fn allocate_pool(&self, memory_type: MemoryType, size: usize) -> Result<*mut u8> {
        let mut ptr = core::ptr::null_mut();
        unsafe { (self.allocate_pool)(memory_type, size, &mut ptr).to_result_with(ptr) }
    }

    /// Libera memória da heap do UEFI.
    pub fn free_pool(&self, ptr: *mut u8) -> Result<()> {
        unsafe { (self.free_pool)(ptr).to_result() }
    }

    /// Localiza um protocolo no sistema (primeiro encontrado).
    pub fn locate_protocol(&self, protocol_guid: &Guid) -> Result<*mut c_void> {
        let mut interface = core::ptr::null_mut();
        unsafe {
            (self.locate_protocol)(protocol_guid, core::ptr::null_mut(), &mut interface)
                .to_result_with(interface)
        }
    }

    /// Abre um protocolo em um handle específico.
    pub fn open_protocol(
        &self,
        handle: Handle,
        protocol: &Guid,
        agent: Handle,
        controller: Handle,
        attributes: u32,
    ) -> Result<*mut c_void> {
        let mut interface = core::ptr::null_mut();
        unsafe {
            (self.open_protocol)(
                handle,
                protocol,
                &mut interface,
                agent,
                controller,
                attributes,
            )
            .to_result_with(interface)
        }
    }

    /// Define o Watchdog Timer. 0 desabilita.
    pub fn set_watchdog_timer(&self, timeout_seconds: usize, watchdog_code: u64) -> Result<()> {
        unsafe {
            (self.set_watchdog_timer)(timeout_seconds, watchdog_code, 0, core::ptr::null())
                .to_result()
        }
    }

    /// Pausa a execução (Busy Wait).
    pub fn stall(&self, microseconds: usize) {
        unsafe {
            let _ = (self.stall)(microseconds);
        }
    }
}
