//! Gerenciamento de ACPI (Advanced Configuration and Power Interface)
//!
//! Responsável por localizar a tabela RSDP (Root System Description Pointer)
//! através das tabelas de configuração do UEFI. Esta é a "chave" que o Kernel
//! precisa para descobrir quantos CPUs existem, controlar energia, etc.

use crate::{
    core::error::{BootError, Result},
    uefi::{
        system_table,
        table::config::{ACPI_20_TABLE_GUID, ACPI_TABLE_GUID},
    },
};

pub struct AcpiManager;

impl AcpiManager {
    /// Localiza o endereço físico do RSDP.
    ///
    /// Prioriza ACPI 2.0 (XSDT) sobre ACPI 1.0 (RSDT) conforme padrão moderno.
    ///
    /// # Retorna
    /// * `Ok(u64)`: Endereço físico do RSDP.
    /// * `Err`: Se nenhuma tabela ACPI for encontrada no firmware.
    pub fn get_rsdp_address() -> Result<u64> {
        let st = system_table();

        // 1. Tentar encontrar ACPI 2.0 (Preferencial em x86_64 e AArch64)
        if let Some(addr) = st.get_configuration_table(&ACPI_20_TABLE_GUID) {
            crate::println!("Hardware: ACPI 2.0 (XSDT) encontrado em {:#p}", addr);
            return Ok(addr as u64);
        }

        // 2. Fallback para ACPI 1.0 (Sistemas Legacy/VMs antigas)
        if let Some(addr) = st.get_configuration_table(&ACPI_TABLE_GUID) {
            crate::println!("Hardware: ACPI 1.0 (RSDT) encontrado em {:#p}", addr);
            return Ok(addr as u64);
        }

        crate::println!("ERRO CRÍTICO: Tabela ACPI não encontrada no firmware.");
        Err(BootError::Generic("ACPI RSDP not found"))
    }
}
