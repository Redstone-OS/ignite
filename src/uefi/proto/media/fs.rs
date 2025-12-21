//! Simple File System Protocol
//!
//! Referência: UEFI Spec 2.10, Seção 13.4

use super::file::FileProtocol;
use crate::uefi::base::*;

/// Simple File System Protocol GUID
pub const SIMPLE_FILE_SYSTEM_PROTOCOL_GUID: Guid = Guid::new(
    0x0964e5b22,
    0x6459,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

/// Simple File System Protocol Revision
pub const SIMPLE_FILE_SYSTEM_PROTOCOL_REVISION: u64 = 0x00010000;

/// Simple File System Protocol
///
/// Spec: 13.4
#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: u64,

    /// Abre o volume raiz do filesystem
    pub open_volume: extern "efiapi" fn(
        *mut Self,
        *mut *mut FileProtocol, // Root
    ) -> Status,
}
