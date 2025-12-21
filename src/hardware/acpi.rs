//! Suporte a Tabelas ACPI
//!
//! Localiza e parseia tabelas ACPI (RSDP, RSDT, XSDT)

use core::mem;

/// Estrutura RSDP (Root System Description Pointer)
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Rsdp {
    pub signature:    [u8; 8], // "RSD PTR "
    pub checksum:     u8,
    pub oem_id:       [u8; 6],
    pub revision:     u8,
    pub rsdt_address: u32,
}

/// RSDP Estendido para ACPI 2.0+
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct RsdpExtended {
    pub rsdp:              Rsdp,
    pub length:            u32,
    pub xsdt_address:      u64,
    pub extended_checksum: u8,
    pub reserved:          [u8; 3],
}

/// Cabeçalho ACPI SDT
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SdtHeader {
    pub signature:        [u8; 4],
    pub length:           u32,
    pub revision:         u8,
    pub checksum:         u8,
    pub oem_id:           [u8; 6],
    pub oem_table_id:     [u8; 8],
    pub oem_revision:     u32,
    pub creator_id:       u32,
    pub creator_revision: u32,
}

/// Gerenciador ACPI
pub struct AcpiManager {
    rsdp_addr: u64,
}

impl AcpiManager {
    /// Encontrar RSDP na memória
    pub fn find_rsdp() -> Option<u64> {
        // TODO: Buscar RSDP em EBDA e áreas da BIOS
        // Em UEFI, obter da Tabela de Sistema EFI
        None
    }

    /// Criar gerenciador ACPI a partir do endereço RSDP
    pub fn new(rsdp_addr: u64) -> Self {
        Self { rsdp_addr }
    }

    /// Obter RSDP
    pub fn get_rsdp(&self) -> &Rsdp {
        unsafe { &*(self.rsdp_addr as *const Rsdp) }
    }

    /// Validar checksum do RSDP
    pub fn validate_rsdp(&self) -> bool {
        let rsdp = self.get_rsdp();
        let bytes = unsafe {
            core::slice::from_raw_parts(rsdp as *const _ as *const u8, mem::size_of::<Rsdp>())
        };

        let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }

    /// Obter endereço RSDT/XSDT
    pub fn get_sdt_address(&self) -> u64 {
        let rsdp = self.get_rsdp();

        if rsdp.revision >= 2 {
            // ACPI 2.0+, usar XSDT
            let extended = unsafe { &*(self.rsdp_addr as *const RsdpExtended) };
            extended.xsdt_address
        } else {
            // ACPI 1.0, usar RSDT
            rsdp.rsdt_address as u64
        }
    }
}
