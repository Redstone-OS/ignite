//! Loaded Image Protocol
//!
//! Referência: UEFI Spec 2.10, Seção 9.1

use crate::uefi::{base::*, table::system::SystemTable};

/// Loaded Image Protocol GUID
pub const LOADED_IMAGE_PROTOCOL_GUID: Guid = Guid::new(
    0x5B1B31A1,
    0x9562,
    0x11d2,
    [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

/// Loaded Image Protocol Revision
pub const LOADED_IMAGE_PROTOCOL_REVISION: u32 = 0x1000;

/// Loaded Image Protocol
///
/// Spec: 9.1
#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision:      u32,
    pub parent_handle: Handle,
    pub system_table:  *mut SystemTable,

    // Source location of image
    pub device_handle: Handle,
    pub file_path:     *mut core::ffi::c_void, // DevicePath
    pub reserved:      *mut core::ffi::c_void,

    // Image's load options
    pub load_options_size: u32,
    pub load_options:      *mut core::ffi::c_void,

    // Location where image was loaded
    pub image_base:      *mut core::ffi::c_void,
    pub image_size:      u64,
    pub image_code_type: u32, // MemoryType
    pub image_data_type: u32, // MemoryType

    pub unload: extern "efiapi" fn(Handle) -> Status,
}
