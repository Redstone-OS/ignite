//! File Protocol
//!
//! Permite manipulação de arquivos (Open, Read, Write, Close).
//! Referência: UEFI Spec 2.10, Seção 13.5

use core::ffi::c_void;

use crate::uefi::{
    Result,
    base::{Char16, Guid, Status},
};

pub const FILE_MODE_READ: u64 = 0x0000000000000001;
pub const FILE_MODE_WRITE: u64 = 0x0000000000000002;
pub const FILE_MODE_CREATE: u64 = 0x8000000000000000;

pub const FILE_INFO_GUID: Guid = Guid::new(
    0x09576e92,
    0x6d3f,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

#[repr(C)]
pub struct FileProtocol {
    pub revision:     u64,
    pub open: extern "efiapi" fn(*mut Self, *mut *mut Self, *const Char16, u64, u64) -> Status,
    pub close:        extern "efiapi" fn(*mut Self) -> Status,
    pub delete:       extern "efiapi" fn(*mut Self) -> Status,
    pub read:         extern "efiapi" fn(*mut Self, *mut usize, *mut c_void) -> Status,
    pub write:        extern "efiapi" fn(*mut Self, *mut usize, *const c_void) -> Status,
    pub get_position: extern "efiapi" fn(*mut Self, *mut u64) -> Status,
    pub set_position: extern "efiapi" fn(*mut Self, u64) -> Status,
    pub get_info:     extern "efiapi" fn(*mut Self, *const Guid, *mut usize, *mut c_void) -> Status,
    pub set_info:     extern "efiapi" fn(*mut Self, *const Guid, usize, *const c_void) -> Status,
    pub flush:        extern "efiapi" fn(*mut Self) -> Status,
}

impl FileProtocol {
    pub fn close_safe(&mut self) -> Result<()> {
        unsafe { (self.close)(self).to_result() }
    }
}
