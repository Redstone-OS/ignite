//! Runtime Services - Serviços de Tempo de Execução
//!
//! Funções que persistem mesmo após o Kernel assumir (se mapeadas
//! corretamente). Referência: UEFI Spec 2.10, Seção 8

use core::ffi::c_void;

use crate::uefi::{
    Result,
    base::{Char16, Guid, Status},
    table::header::TableHeader,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Time {
    pub year:       u16, // 1900 - 9999
    pub month:      u8,  // 1 - 12
    pub day:        u8,  // 1 - 31
    pub hour:       u8,  // 0 - 23
    pub minute:     u8,  // 0 - 59
    pub second:     u8,  // 0 - 59
    pub pad1:       u8,
    pub nanosecond: u32,
    pub time_zone:  i16, // -1440 to 1440
    pub daylight:   u8,
    pub pad2:       u8,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ResetType {
    Cold = 0,
    Warm = 1,
    Shutdown = 2,
    PlatformSpecific = 3,
}

#[repr(C)]
pub struct RuntimeServices {
    pub hdr: TableHeader,

    // Time Services
    pub get_time:        extern "efiapi" fn(*mut Time, *mut c_void) -> Status,
    pub set_time:        extern "efiapi" fn(*mut Time) -> Status,
    pub get_wakeup_time: extern "efiapi" fn(*mut u8, *mut u8, *mut Time) -> Status,
    pub set_wakeup_time: extern "efiapi" fn(u8, *mut Time) -> Status,

    // Virtual Memory Services
    pub set_virtual_address_map: extern "efiapi" fn(usize, usize, u32, *mut c_void) -> Status,
    pub convert_pointer:         extern "efiapi" fn(usize, *mut *mut c_void) -> Status,

    // Variable Services
    pub get_variable:
        extern "efiapi" fn(*const Char16, *const Guid, *mut u32, *mut usize, *mut c_void) -> Status,
    pub get_next_variable_name: extern "efiapi" fn(*mut usize, *mut Char16, *mut Guid) -> Status,
    pub set_variable:
        extern "efiapi" fn(*const Char16, *const Guid, u32, usize, *mut c_void) -> Status,

    // Miscellaneous Services
    pub get_next_high_monotonic_count: extern "efiapi" fn(*mut u32) -> Status,
    pub reset_system: extern "efiapi" fn(ResetType, Status, usize, *const c_void) -> !,

    // UEFI 2.0 Capsule Services
    pub update_capsule:             extern "efiapi" fn(*mut *mut c_void, usize, u64) -> Status,
    pub query_capsule_capabilities:
        extern "efiapi" fn(*mut *mut c_void, usize, *mut u64, *mut u32) -> Status,
    pub query_variable_info:        extern "efiapi" fn(u32, *mut u64, *mut u64, *mut u64) -> Status,
}

impl RuntimeServices {
    /// Reinicia ou desliga o sistema.
    pub fn reset_system(&self, ty: ResetType, status: Status) -> ! {
        unsafe {
            (self.reset_system)(ty, status, 0, core::ptr::null());
        }
    }

    /// Obtém a data e hora atual do Hardware (RTC).
    pub fn get_time(&self) -> Result<Time> {
        let mut time = Time::default();
        unsafe { (self.get_time)(&mut time, core::ptr::null_mut()).to_result_with(time) }
    }
}
