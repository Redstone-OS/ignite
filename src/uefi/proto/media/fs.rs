//! Simple File System Protocol
//!
//! Permite abrir o volume raiz de um dispositivo de bloco.
//! Referência: UEFI Spec 2.10, Seção 13.4

use super::file::FileProtocol;
use crate::uefi::{
    Result,
    base::{Guid, Status},
};

pub const SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: Guid = Guid::new(
    0x0964e5b22,
    0x6459,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision:    u64,
    pub open_volume: extern "efiapi" fn(*mut Self, *mut *mut FileProtocol) -> Status,
}

impl SimpleFileSystemProtocol {
    pub fn open_volume(&mut self) -> Result<&mut FileProtocol> {
        let mut file_ptr = core::ptr::null_mut();
        unsafe {
            (self.open_volume)(self, &mut file_ptr)
                .to_result_with(file_ptr)
                .map(|ptr| &mut *ptr)
        }
    }
}
