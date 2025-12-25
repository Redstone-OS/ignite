//! # ACPI Discovery (RSDP Finder)
//!
//! Este módulo é o **Pathfinder da Configuração de Hardware**.
//! Sua única responsabilidade é encontrar o "Ponteiro Sagrado" (RSDP) na
//! memória, que servirá de âncora para o Kernel descobrir toda a topologia da
//! máquina (CPUs, IOAPIC, HPET).
//!
//! ## 🎯 Mecânica de Descoberta
//! A UEFI simplifica drasticamente isso em comparação com a BIOS (onde
//! precisávamos escanear o EBDA). A System Table da UEFI expõe o RSDP como uma
//! "Configuration Table", identificada por GUIDs.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Modernidade:** Prioriza ACPI 2.0 (`ACPI_20_TABLE_GUID`). Isso garante
//!   acesso a XSDT (endereços 64-bit).
//! - **Segurança de Tipo:** Usa GUIDs tipados da crate `uefi`.
//!
//! ### ⚠️ Pontos de Atenção (Riscos)
//! - **Confiança Cega:** O módulo retorna o endereço sem validar o Checksum do
//!   RSDP.
//!   - *Risco:* Se a BIOS estiver bugada e apontar para lixo, o Kernel vai
//!     travar ao tentar parsear.
//! - **Sem Leitura:** O Bootloader não lê as tabelas, apenas passa o ponteiro.
//!   Isso é bom (mantém bootloader simples) e ruim (perde chance de validar
//!   cedo).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Reliability)** Validar Checksum do RSDP antes de aceitar.
//!   - *Motivo:* Fail-fast. Se o RSDP estiver corrompido, avisar o usuário
//!     antes de bootar o kernel.
//! - [ ] **TODO: (Feature)** Dump básico da topologia para debug.
//!   - *Idea:* Imprimir "Found X CPUs" se `ignite.cfg` tiver `debug=true`.

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
