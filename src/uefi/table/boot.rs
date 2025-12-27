//! Boot Services - Serviços de Inicialização
//!
//! Contém as definições da tabela de serviços de boot e implementações seguras
//! para alocação de memória, manipulação de protocolos e eventos.
//!
//! OBSERVAÇÃO: Os campos da struct `BootServices` (ponteiros de função) possuem
//! sufixo `_f` para não colidir com os métodos seguros implementados no `impl`.

use core::ffi::c_void;

use crate::uefi::{
    base::{Char16, Event, Guid, Handle, Status},
    table::header::TableHeader,
    Result,
};

// --- Tipos de Enumeração ---

/// Chave do mapa de memória (usada em ExitBootServices).
pub type MemoryMapKey = usize;

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
    pub raise_tpl_f:   unsafe extern "efiapi" fn(usize) -> usize,
    pub restore_tpl_f: unsafe extern "efiapi" fn(usize),

    // Memory Services
    pub allocate_pages_f:
        unsafe extern "efiapi" fn(AllocateType, MemoryType, usize, *mut u64) -> Status,
    pub free_pages_f:     unsafe extern "efiapi" fn(u64, usize) -> Status,
    pub get_memory_map_f: unsafe extern "efiapi" fn(
        *mut usize,
        *mut MemoryDescriptor,
        *mut usize,
        *mut usize,
        *mut u32,
    ) -> Status,
    pub allocate_pool_f:  unsafe extern "efiapi" fn(MemoryType, usize, *mut *mut u8) -> Status,
    pub free_pool_f:      unsafe extern "efiapi" fn(*mut u8) -> Status,

    // Event & Timer Services
    pub create_event_f:
        unsafe extern "efiapi" fn(u32, usize, usize, *mut c_void, *mut Event) -> Status,
    pub set_timer_f:      unsafe extern "efiapi" fn(Event, TimerDelay, u64) -> Status,
    pub wait_for_event_f: unsafe extern "efiapi" fn(usize, *mut Event, *mut usize) -> Status,
    pub signal_event_f:   unsafe extern "efiapi" fn(Event) -> Status,
    pub close_event_f:    unsafe extern "efiapi" fn(Event) -> Status,
    pub check_event_f:    unsafe extern "efiapi" fn(Event) -> Status,

    // Protocol Handler Services
    pub install_protocol_interface_f:
        unsafe extern "efiapi" fn(*mut Handle, *const Guid, InterfaceType, *mut c_void) -> Status,
    pub reinstall_protocol_interface_f:
        unsafe extern "efiapi" fn(Handle, *const Guid, *mut c_void, *mut c_void) -> Status,
    pub uninstall_protocol_interface_f:
        unsafe extern "efiapi" fn(Handle, *const Guid, *mut c_void) -> Status,
    pub handle_protocol_f:
        unsafe extern "efiapi" fn(Handle, *const Guid, *mut *mut c_void) -> Status,
    pub reserved: *mut c_void,
    pub register_protocol_notify_f:
        unsafe extern "efiapi" fn(*const Guid, Event, *mut *mut c_void) -> Status,
    pub locate_handle_f: unsafe extern "efiapi" fn(
        LocateSearchType,
        *const Guid,
        *mut c_void,
        *mut usize,
        *mut Handle,
    ) -> Status,
    pub locate_device_path_f:
        unsafe extern "efiapi" fn(*const Guid, *mut *mut c_void, *mut Handle) -> Status,
    pub install_configuration_table_f:
        unsafe extern "efiapi" fn(*const Guid, *mut c_void) -> Status,

    // Image Services
    pub load_image_f: unsafe extern "efiapi" fn(
        u8,
        Handle,
        *mut c_void,
        *mut c_void,
        usize,
        *mut Handle,
    ) -> Status,
    pub start_image_f: unsafe extern "efiapi" fn(Handle, *mut usize, *mut *mut u16) -> Status,
    pub exit_f:               unsafe extern "efiapi" fn(Handle, Status, usize, *mut u16) -> Status,
    pub unload_image_f:       unsafe extern "efiapi" fn(Handle) -> Status,
    pub exit_boot_services_f: unsafe extern "efiapi" fn(Handle, usize) -> Status,

    // Miscellaneous Services
    pub get_next_monotonic_count_f: unsafe extern "efiapi" fn(*mut u64) -> Status,
    pub stall_f:                    unsafe extern "efiapi" fn(usize) -> Status,
    pub set_watchdog_timer_f: unsafe extern "efiapi" fn(usize, u64, usize, *const Char16) -> Status,

    // Driver Support Services
    pub connect_controller_f:
        unsafe extern "efiapi" fn(Handle, *mut Handle, *mut c_void, u8) -> Status,
    pub disconnect_controller_f: unsafe extern "efiapi" fn(Handle, Handle, Handle) -> Status,

    // Open and Close Protocol Services
    pub open_protocol_f: unsafe extern "efiapi" fn(
        Handle,
        *const Guid,
        *mut *mut c_void,
        Handle,
        Handle,
        u32,
    ) -> Status,
    pub close_protocol_f: unsafe extern "efiapi" fn(Handle, *const Guid, Handle, Handle) -> Status,
    pub open_protocol_information_f:
        unsafe extern "efiapi" fn(Handle, *const Guid, *mut *mut c_void, *mut usize) -> Status,

    // Library Services
    pub protocols_per_handle_f:
        unsafe extern "efiapi" fn(Handle, *mut *mut *mut Guid, *mut usize) -> Status,
    pub locate_handle_buffer_f: unsafe extern "efiapi" fn(
        LocateSearchType,
        *const Guid,
        *mut c_void,
        *mut usize,
        *mut *mut Handle,
    ) -> Status,
    pub locate_protocol_f:
        unsafe extern "efiapi" fn(*const Guid, *mut c_void, *mut *mut c_void) -> Status,
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
        // Nota: Para AllocateAddress, addr deve ser preenchido pelo chamador.
        // Se usar AllocateAddress, certifique-se de chamar allocate_at ou configurar
        // addr.
        unsafe { (self.allocate_pages_f)(ty, memory_type, pages, &mut addr).to_result_with(addr) }
    }

    /// Aloca páginas em um endereço específico.
    pub fn allocate_at(
        &self,
        memory_type: MemoryType,
        pages: usize,
        target_addr: u64,
    ) -> Result<u64> {
        let mut addr = target_addr;
        unsafe {
            (self.allocate_pages_f)(AllocateType::AllocateAddress, memory_type, pages, &mut addr)
                .to_result_with(addr)
        }
    }

    /// Libera páginas de memória.
    pub fn free_pages(&self, addr: u64, pages: usize) -> Result<()> {
        unsafe { (self.free_pages_f)(addr, pages).to_result() }
    }

    /// Aloca memória na heap do UEFI (Pool).
    pub fn allocate_pool(&self, memory_type: MemoryType, size: usize) -> Result<*mut u8> {
        let mut ptr = core::ptr::null_mut();
        unsafe { (self.allocate_pool_f)(memory_type, size, &mut ptr).to_result_with(ptr) }
    }

    /// Libera memória da heap do UEFI.
    pub fn free_pool(&self, ptr: *mut u8) -> Result<()> {
        unsafe { (self.free_pool_f)(ptr).to_result() }
    }

    /// Localiza um protocolo no sistema (primeiro encontrado).
    pub fn locate_protocol(&self, protocol_guid: &Guid) -> Result<*mut c_void> {
        let mut interface = core::ptr::null_mut();
        unsafe {
            (self.locate_protocol_f)(protocol_guid, core::ptr::null_mut(), &mut interface)
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
            (self.open_protocol_f)(
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
            (self.set_watchdog_timer_f)(timeout_seconds, watchdog_code, 0, core::ptr::null())
                .to_result()
        }
    }

    /// Pausa a execução (Busy Wait).
    pub fn stall(&self, microseconds: usize) {
        unsafe {
            let _ = (self.stall_f)(microseconds);
        }
    }

    /// Sai dos serviços de boot.
    pub fn exit_boot_services(&self, image_handle: Handle, map_key: usize) -> Status {
        unsafe { (self.exit_boot_services_f)(image_handle, map_key) }
    }
}
