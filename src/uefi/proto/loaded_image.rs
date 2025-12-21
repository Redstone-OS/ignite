//! Protocolo Loaded Image
//!
//! Fornece informações sobre a imagem UEFI carregada (o próprio bootloader),
//! como o dispositivo de origem e o tamanho da memória.
//! Referência: UEFI Spec 2.10, Seção 9.1

use core::ffi::c_void;

use crate::uefi::{
    base::{Guid, Handle, Status},
    table::system::SystemTable,
};

pub const LOADED_IMAGE_PROTOCOL_GUID: Guid = Guid::new(
    0x5B1B31A1,
    0x9562,
    0x11d2,
    [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
);

#[repr(C)]
pub struct LoadedImageProtocol {
    pub revision:      u32,
    pub parent_handle: Handle,
    pub system_table:  *mut SystemTable,

    // Dispositivo de onde a imagem foi carregada
    pub device_handle: Handle,
    pub file_path:     *mut c_void, // DevicePathProtocol
    pub reserved:      *mut c_void,

    // Opções de carregamento da imagem
    pub load_options_size: u32,
    pub load_options:      *mut c_void,

    // Localização da imagem na memória
    pub image_base:      *mut c_void,
    pub image_size:      u64,
    pub image_code_type: u32, // MemoryType
    pub image_data_type: u32, // MemoryType

    pub unload: extern "efiapi" fn(Handle) -> Status,
}
