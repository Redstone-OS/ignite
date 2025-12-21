//! EFI System Table
//!
//! Referência: UEFI Spec 2.10, Seção 4.3

use super::{boot::BootServices, runtime::RuntimeServices};
use crate::uefi::base::*;

/// EFI Table Header
///
/// Spec: 4.2 - EFI Table Header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TableHeader {
    /// Assinatura única da tabela (0x5453595320494249 para System Table)
    pub signature:   u64,
    /// Revisão da especificação UEFI
    pub revision:    u32,
    /// Tamanho do header em bytes
    pub header_size: u32,
    /// CRC32 do header e tabela
    pub crc32:       u32,
    /// Reservado, deve ser 0
    pub reserved:    u32,
}

/// Simple Text Input Protocol
///
/// Spec: 12.3 - Simple Text Input Protocol
#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset:           extern "efiapi" fn(*mut Self, Boolean) -> Status,
    pub read_key_stroke: extern "efiapi" fn(*mut Self, *mut InputKey) -> Status,
    pub wait_for_key:    Event,
}

impl SimpleTextInputProtocol {
    /// Helper para ler uma tecla (wraps read_key_stroke)
    pub fn read(&mut self) -> Result<Option<InputKey>> {
        let mut key = InputKey {
            scan_code:    0,
            unicode_char: 0,
        };
        let status = (self.read_key_stroke)(self as *mut _, &mut key);

        match status {
            Status::SUCCESS => Ok(Some(key)),
            Status::NOT_READY => Ok(None),
            _ => Err(status),
        }
    }
}

/// Input Key
///
/// Spec: 12.3.3 - EFI_INPUT_KEY
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InputKey {
    pub scan_code:    u16,
    pub unicode_char: Char16,
}

// Scan Codes - Spec: 12.3.4
pub const SCAN_NULL: u16 = 0x00;
pub const SCAN_UP: u16 = 0x01;
pub const SCAN_DOWN: u16 = 0x02;
pub const SCAN_RIGHT: u16 = 0x03;
pub const SCAN_LEFT: u16 = 0x04;
pub const SCAN_HOME: u16 = 0x05;
pub const SCAN_END: u16 = 0x06;
pub const SCAN_INSERT: u16 = 0x07;
pub const SCAN_DELETE: u16 = 0x08;
pub const SCAN_PAGE_UP: u16 = 0x09;
pub const SCAN_PAGE_DOWN: u16 = 0x0A;
pub const SCAN_F1: u16 = 0x0B;
pub const SCAN_F2: u16 = 0x0C;
pub const SCAN_ESC: u16 = 0x17;

/// Simple Text Output Protocol
///
/// Spec: 12.4 - Simple Text Output Protocol
#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset:               extern "efiapi" fn(*mut Self, Boolean) -> Status,
    pub output_string:       extern "efiapi" fn(*mut Self, *const Char16) -> Status,
    pub test_string:         extern "efiapi" fn(*mut Self, *const Char16) -> Status,
    pub query_mode:          extern "efiapi" fn(*mut Self, usize, *mut usize, *mut usize) -> Status,
    pub set_mode:            extern "efiapi" fn(*mut Self, usize) -> Status,
    pub set_attribute:       extern "efiapi" fn(*mut Self, usize) -> Status,
    pub clear_screen:        extern "efiapi" fn(*mut Self) -> Status,
    pub set_cursor_position: extern "efiapi" fn(*mut Self, usize, usize) -> Status,
    pub enable_cursor:       extern "efiapi" fn(*mut Self, Boolean) -> Status,
    pub mode:                *mut SimpleTextOutputMode,
}

#[repr(C)]
pub struct SimpleTextOutputMode {
    pub max_mode:       i32,
    pub mode:           i32,
    pub attribute:      i32,
    pub cursor_column:  i32,
    pub cursor_row:     i32,
    pub cursor_visible: Boolean,
}

/// Configuration Table Entry
///
/// Spec: 4.6 - EFI Configuration Table
#[repr(C)]
pub struct ConfigurationTable {
    pub vendor_guid:  Guid,
    pub vendor_table: *mut core::ffi::c_void,
}

/// EFI System Table
///
/// Spec: 4.3 - EFI System Table
#[repr(C)]
pub struct SystemTable {
    /// EFI Table Header
    pub hdr:                     TableHeader,
    /// Ponteiro para string Unicode com nome do firmware vendor
    pub firmware_vendor:         *const Char16,
    /// Revisão do firmware
    pub firmware_revision:       u32,
    /// Handle para console input device
    pub console_in_handle:       Handle,
    /// Ponteiro para Simple Text Input Protocol
    pub con_in:                  *mut SimpleTextInputProtocol,
    /// Handle para console output device
    pub console_out_handle:      Handle,
    /// Ponteiro para Simple Text Output Protocol
    pub con_out:                 *mut SimpleTextOutputProtocol,
    /// Handle para standard error device
    pub standard_error_handle:   Handle,
    /// Ponteiro para Simple Text Output Protocol para stderr
    pub std_err:                 *mut SimpleTextOutputProtocol,
    /// Ponteiro para EFI Runtime Services Table
    pub runtime_services:        *mut RuntimeServices,
    /// Ponteiro para EFI Boot Services Table
    pub boot_services:           *mut BootServices,
    /// Número de entradas na configuration table
    pub number_of_table_entries: usize,
    /// Ponteiro para configuration table
    pub configuration_table:     *mut ConfigurationTable,
}

// GUIDs importantes para Configuration Tables

/// ACPI 2.0+ Table GUID
pub const ACPI_20_TABLE_GUID: Guid = Guid::new(
    0x8868e871,
    0xe4f1,
    0x11d3,
    [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
);

/// ACPI 1.0 Table GUID
pub const ACPI_TABLE_GUID: Guid = Guid::new(
    0xeb9d2d30,
    0x2d88,
    0x11d3,
    [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
);

/// SMBIOS Table GUID
pub const SMBIOS_TABLE_GUID: Guid = Guid::new(
    0xeb9d2d31,
    0x2d88,
    0x11d3,
    [0x9a, 0x16, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
);

/// SMBIOS 3.0 Table GUID
pub const SMBIOS3_TABLE_GUID: Guid = Guid::new(
    0xf2fd1544,
    0x9794,
    0x4a2c,
    [0x99, 0x2e, 0xe5, 0xbb, 0xcf, 0x20, 0xe3, 0x94],
);
