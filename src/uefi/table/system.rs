//! Tabela do Sistema (System Table)
//!
//! O ponto de entrada global para todos os serviços UEFI.
//! Contém ponteiros para as tabelas de Runtime e Boot Services, além dos
//! protocolos de Console. Referência: UEFI Spec 2.10, Seção 4.3

use core::ffi::c_void;

use crate::uefi::{
    base::{Boolean, Char16, Event, Guid, Handle, Status},
    table::{boot::BootServices, header::TableHeader, runtime::RuntimeServices},
};

// --- Protocolos de Console (Input/Output) ---

/// Tecla de Entrada (Input Key).
/// Referência: UEFI Spec 12.3.3
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct InputKey {
    pub scan_code:    u16,
    pub unicode_char: Char16,
}

/// Protocolo de Entrada de Texto Simples.
/// Referência: UEFI Spec 12.3
#[repr(C)]
pub struct SimpleTextInputProtocol {
    pub reset:           extern "efiapi" fn(*mut SimpleTextInputProtocol, Boolean) -> Status,
    pub read_key_stroke: extern "efiapi" fn(*mut SimpleTextInputProtocol, *mut InputKey) -> Status,
    pub wait_for_key:    Event,
}

/// Modo do Protocolo de Saída de Texto.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SimpleTextOutputMode {
    pub max_mode:       i32,
    pub mode:           i32,
    pub attribute:      i32,
    pub cursor_column:  i32,
    pub cursor_row:     i32,
    pub cursor_visible: Boolean,
}

/// Protocolo de Saída de Texto Simples.
/// Referência: UEFI Spec 12.4
#[repr(C)]
pub struct SimpleTextOutputProtocol {
    pub reset:               extern "efiapi" fn(*mut SimpleTextOutputProtocol, Boolean) -> Status,
    pub output_string: extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const Char16) -> Status,
    pub test_string: extern "efiapi" fn(*mut SimpleTextOutputProtocol, *const Char16) -> Status,
    pub query_mode:
        extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize, *mut usize, *mut usize) -> Status,
    pub set_mode:            extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize) -> Status,
    pub set_attribute:       extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize) -> Status,
    pub clear_screen:        extern "efiapi" fn(*mut SimpleTextOutputProtocol) -> Status,
    pub set_cursor_position:
        extern "efiapi" fn(*mut SimpleTextOutputProtocol, usize, usize) -> Status,
    pub enable_cursor:       extern "efiapi" fn(*mut SimpleTextOutputProtocol, Boolean) -> Status,
    pub mode:                *mut SimpleTextOutputMode,
}

// --- Estruturas da System Table ---

/// Entrada na Tabela de Configuração (Vendor Table).
/// Usada para localizar ACPI (RSDP), SMBIOS, Device Tree, etc.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ConfigurationTable {
    pub vendor_guid:  Guid,
    pub vendor_table: *mut c_void,
}

/// A Tabela do Sistema UEFI.
#[repr(C)]
pub struct SystemTable {
    pub hdr:               TableHeader,
    pub firmware_vendor:   *const Char16,
    pub firmware_revision: u32,

    pub console_in_handle: Handle,
    pub con_in:            *mut SimpleTextInputProtocol,

    pub console_out_handle: Handle,
    pub con_out:            *mut SimpleTextOutputProtocol,

    pub standard_error_handle: Handle,
    pub std_err:               *mut SimpleTextOutputProtocol,

    pub runtime_services: *mut RuntimeServices,
    pub boot_services:    *mut BootServices,

    pub number_of_table_entries: usize,
    pub configuration_table:     *mut ConfigurationTable,
}

impl SystemTable {
    /// Acessa os serviços de boot.
    ///
    /// # Safety
    /// Retorna referência 'static. Válido APENAS enquanto `ExitBootServices`
    /// não for chamado.
    pub fn boot_services(&self) -> &'static BootServices {
        unsafe { &*self.boot_services }
    }

    /// Acessa os serviços de runtime.
    ///
    /// # Safety
    /// Válido durante todo o ciclo de vida do sistema (boot e OS), desde que o
    /// mapeamento virtual seja respeitado.
    pub fn runtime_services(&self) -> &'static RuntimeServices {
        unsafe { &*self.runtime_services }
    }

    /// Procura uma tabela de configuração pelo GUID (ex: ACPI).
    ///
    /// # Argumentos
    /// * `guid` - O GUID da tabela desejada (ex: `config::ACPI_20_TABLE_GUID`).
    ///
    /// # Retorna
    /// `Option<*mut c_void>` apontando para a estrutura física da tabela.
    pub fn get_configuration_table(&self, guid: &Guid) -> Option<*mut c_void> {
        let tables = unsafe {
            core::slice::from_raw_parts(self.configuration_table, self.number_of_table_entries)
        };

        for table in tables {
            if table.vendor_guid == *guid {
                return Some(table.vendor_table);
            }
        }
        None
    }
}
