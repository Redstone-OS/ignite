//! File Protocol
//!
//! Referência: UEFI Spec 2.10, Section 13.5

use crate::uefi::base::*;

/// File Protocol Revision
pub const FILE_PROTOCOL_REVISION: u64 = 0x00010000;
pub const FILE_PROTOCOL_REVISION2: u64 = 0x00020000;
pub const FILE_PROTOCOL_LATEST_REVISION: u64 = FILE_PROTOCOL_REVISION2;

/// File Mode Flags
pub const FILE_MODE_READ: u64 = 0x0000000000000001;
pub const FILE_MODE_WRITE: u64 = 0x0000000000000002;
pub const FILE_MODE_CREATE: u64 = 0x8000000000000000;

/// File Attribute Flags
pub const FILE_READ_ONLY: u64 = 0x0000000000000001;
pub const FILE_HIDDEN: u64 = 0x0000000000000002;
pub const FILE_SYSTEM: u64 = 0x0000000000000004;
pub const FILE_RESERVED: u64 = 0x0000000000000008;
pub const FILE_DIRECTORY: u64 = 0x0000000000000010;
pub const FILE_ARCHIVE: u64 = 0x0000000000000020;
pub const FILE_VALID_ATTR: u64 = 0x0000000000000037;

/// File Info GUID
pub const FILE_INFO_GUID: Guid = Guid::new(
    0x09576e92,
    0x6d3f,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

/// File System Info GUID
pub const FILE_SYSTEM_INFO_GUID: Guid = Guid::new(
    0x09576e93,
    0x6d3f,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

/// EFI Time
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Time {
    pub year:       u16, // 1900 - 9999
    pub month:      u8,  // 1 - 12
    pub day:        u8,  // 1 - 31
    pub hour:       u8,  // 0 - 23
    pub minute:     u8,  // 0 - 59
    pub second:     u8,  // 0 - 59
    pub pad1:       u8,
    pub nanosecond: u32, // 0 - 999,999,999
    pub time_zone:  i16, // -1440 to 1440 or 2047
    pub daylight:   u8,
    pub pad2:       u8,
}

/// File Info
///
/// IMPORTANTE: FileName é variable-length no final da struct
#[repr(C)]
pub struct FileInfo {
    pub size:              u64,
    pub file_size:         u64,
    pub physical_size:     u64,
    pub create_time:       Time,
    pub last_access_time:  Time,
    pub modification_time: Time,
    pub attribute:         u64,
    // FileName: Char16[] começa aqui (variable length)
}

impl FileInfo {
    /// Obtém o nome do arquivo (unsafe - precisa garantir que buffer é válido)
    pub unsafe fn file_name(&self) -> *const Char16 {
        let base = self as *const _ as *const u8;
        let name_offset = core::mem::size_of::<FileInfo>();
        base.add(name_offset) as *const Char16
    }
}

/// File System Info
#[repr(C)]
pub struct FileSystemInfo {
    pub size:        u64,
    pub read_only:   Boolean,
    pub volume_size: u64,
    pub free_space:  u64,
    pub block_size:  u32,
    // VolumeLabel: Char16[] começa aqui (variable length)
}

/// File Protocol
///
/// Spec: 13.5
#[repr(C)]
pub struct FileProtocol {
    pub revision: u64,

    /// Abre um novo arquivo relativo a este arquivo
    pub open: extern "efiapi" fn(
        *mut Self,
        *mut *mut Self, // NewHandle
        *const Char16,  // FileName
        u64,            // OpenMode
        u64,            // Attributes
    ) -> Status,

    /// Fecha este arquivo
    pub close: extern "efiapi" fn(*mut Self) -> Status,

    /// Deleta este arquivo
    pub delete: extern "efiapi" fn(*mut Self) -> Status,

    /// Lê dados do arquivo
    pub read: extern "efiapi" fn(
        *mut Self,
        *mut usize,             // BufferSize (in/out)
        *mut core::ffi::c_void, // Buffer
    ) -> Status,

    /// Escreve dados no arquivo
    pub write: extern "efiapi" fn(
        *mut Self,
        *mut usize,               // BufferSize (in/out)
        *const core::ffi::c_void, // Buffer
    ) -> Status,

    /// Obtém posição atual no arquivo
    pub get_position: extern "efiapi" fn(
        *mut Self,
        *mut u64, // Position
    ) -> Status,

    /// Define posição no arquivo
    pub set_position: extern "efiapi" fn(
        *mut Self,
        u64, // Position
    ) -> Status,

    /// Obtém informações sobre o arquivo
    pub get_info: extern "efiapi" fn(
        *mut Self,
        *const Guid,            // InformationType
        *mut usize,             // BufferSize (in/out)
        *mut core::ffi::c_void, // Buffer
    ) -> Status,

    /// Define informações do arquivo
    pub set_info: extern "efiapi" fn(
        *mut Self,
        *const Guid,              // InformationType
        usize,                    // BufferSize
        *const core::ffi::c_void, // Buffer
    ) -> Status,

    /// Flush dados pendentes
    pub flush: extern "efiapi" fn(*mut Self) -> Status,

    // UEFI 2.0+
    pub open_ex:  usize, // Não implementado ainda
    pub read_ex:  usize, // Não implementado ainda
    pub write_ex: usize, // Não implementado ainda
    pub flush_ex: usize, // Não implementado ainda
}

// ============================================================================
// Helper Types & Implementations
// ============================================================================

/// CStr16 - UTF-16 null-terminated string wrapper
pub struct CStr16;

impl CStr16 {
    /// Creates a CStr16 from a Rust string, writing to provided buffer
    pub fn from_str_with_buf<'a>(
        s: &str,
        buf: &'a mut [u16],
    ) -> core::result::Result<&'a [u16], ()> {
        let mut i = 0;
        for ch in s.chars() {
            if i >= buf.len() - 1 {
                return Err(()); // Buffer too small
            }
            if ch as u32 > 0xFFFF {
                return Err(()); // Character out of range
            }
            buf[i] = ch as u16;
            i += 1;
        }
        buf[i] = 0; // Null terminator
        Ok(&buf[..=i])
    }
}

/// File Mode
#[derive(Debug, Copy, Clone)]
pub enum FileMode {
    Read,
    ReadWrite,
    Create,
}

impl FileMode {
    pub fn as_u64(self) -> u64 {
        match self {
            FileMode::Read => FILE_MODE_READ,
            FileMode::ReadWrite => FILE_MODE_READ | FILE_MODE_WRITE,
            FileMode::Create => FILE_MODE_READ | FILE_MODE_WRITE | FILE_MODE_CREATE,
        }
    }
}

/// File Attribute
#[derive(Debug, Copy, Clone)]
pub struct FileAttribute(pub u64);

impl FileAttribute {
    pub fn empty() -> Self {
        FileAttribute(0)
    }

    pub fn read_only() -> Self {
        FileAttribute(FILE_READ_ONLY)
    }
}
