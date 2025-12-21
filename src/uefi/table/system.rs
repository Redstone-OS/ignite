//! Tabela do Sistema (System Table)
//!
//! O ponto de entrada global para todos os serviços UEFI.
//! Referência: UEFI Spec 2.10, Seção 4.3

use crate::uefi::{
    base::{Char16, Guid, Handle},
    table::{boot::BootServices, header::TableHeader, runtime::RuntimeServices},
};

/// Entrada na Tabela de Configuração (Vendor Table).
/// Usada para localizar ACPI (RSDP), SMBIOS, Device Tree, etc.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ConfigurationTable {
    pub vendor_guid:  Guid,
    pub vendor_table: *mut core::ffi::c_void,
}

/// A Tabela do Sistema UEFI.
#[repr(C)]
pub struct SystemTable {
    pub hdr:               TableHeader,
    pub firmware_vendor:   *const Char16,
    pub firmware_revision: u32,

    pub console_in_handle: Handle,
    pub con_in:            *mut core::ffi::c_void, // Placeholder: SimpleTextInputProtocol

    pub console_out_handle: Handle,
    pub con_out:            *mut core::ffi::c_void, // Placeholder: SimpleTextOutputProtocol

    pub standard_error_handle: Handle,
    pub std_err:               *mut core::ffi::c_void, // Placeholder: SimpleTextOutputProtocol

    pub runtime_services: *mut RuntimeServices,
    pub boot_services:    *mut BootServices,

    pub number_of_table_entries: usize,
    pub configuration_table:     *mut ConfigurationTable,
}

impl SystemTable {
    /// Acessa os serviços de boot.
    pub fn boot_services(&self) -> &'static BootServices {
        unsafe { &*self.boot_services }
    }

    /// Acessa os serviços de runtime.
    pub fn runtime_services(&self) -> &'static RuntimeServices {
        unsafe { &*self.runtime_services }
    }

    /// Procura uma tabela de configuração pelo GUID (ex: ACPI).
    pub fn get_configuration_table(&self, guid: &Guid) -> Option<*mut core::ffi::c_void> {
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
