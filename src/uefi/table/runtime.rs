//! EFI Runtime Services
//!
//! Referência: UEFI Spec 2.10, Seção 8 - Services - Runtime Services

use super::system::TableHeader;
use crate::uefi::base::*;

/// EFI Runtime Services Table
///
/// Spec: 4.5 - EFI Runtime Services Table
#[repr(C)]
pub struct RuntimeServices {
    pub hdr: TableHeader,

    // Time Services
    pub get_time:        usize,
    pub set_time:        usize,
    pub get_wakeup_time: usize,
    pub set_wakeup_time: usize,

    // Virtual Memory Services
    pub set_virtual_address_map: usize,
    pub convert_pointer:         usize,

    // Variable Services
    pub get_variable:           usize,
    pub get_next_variable_name: usize,
    pub set_variable:           usize,

    // Miscellaneous Services
    pub get_next_high_monotonic_count: usize,
    pub reset_system: extern "efiapi" fn(u32, Status, usize, *const core::ffi::c_void) -> !,

    // UEFI 2.0+ Capsule Services
    pub update_capsule:             usize,
    pub query_capsule_capabilities: usize,

    // Miscellaneous UEFI 2.0+ Service
    pub query_variable_info: usize,
}

/// Reset Type
#[repr(u32)]
pub enum ResetType {
    ResetCold = 0,
    ResetWarm = 1,
    ResetShutdown = 2,
    ResetPlatformSpecific = 3,
}
