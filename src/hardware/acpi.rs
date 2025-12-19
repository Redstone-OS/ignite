//! ACPI Table Support
//!
//! Locate and parse ACPI tables (RSDP, RSDT, XSDT)

use core::mem;

/// RSDP (Root System Description Pointer) structure
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Rsdp {
    pub signature:    [u8; 8], // "RSD PTR "
    pub checksum:     u8,
    pub oem_id:       [u8; 6],
    pub revision:     u8,
    pub rsdt_address: u32,
}

/// Extended RSDP for ACPI 2.0+
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct RsdpExtended {
    pub rsdp:              Rsdp,
    pub length:            u32,
    pub xsdt_address:      u64,
    pub extended_checksum: u8,
    pub reserved:          [u8; 3],
}

/// ACPI SDT Header
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

/// ACPI manager
pub struct AcpiManager {
    rsdp_addr: u64,
}

impl AcpiManager {
    /// Find RSDP in memory
    pub fn find_rsdp() -> Option<u64> {
        // TODO: Search for RSDP in EBDA and BIOS areas
        // On UEFI, get from EFI System Table
        None
    }

    /// Create ACPI manager from RSDP address
    pub fn new(rsdp_addr: u64) -> Self {
        Self { rsdp_addr }
    }

    /// Get RSDP
    pub fn get_rsdp(&self) -> &Rsdp {
        unsafe { &*(self.rsdp_addr as *const Rsdp) }
    }

    /// Validate RSDP checksum
    pub fn validate_rsdp(&self) -> bool {
        let rsdp = self.get_rsdp();
        let bytes = unsafe {
            core::slice::from_raw_parts(rsdp as *const _ as *const u8, mem::size_of::<Rsdp>())
        };

        let sum: u8 = bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        sum == 0
    }

    /// Get RSDT/XSDT address
    pub fn get_sdt_address(&self) -> u64 {
        let rsdp = self.get_rsdp();

        if rsdp.revision >= 2 {
            // ACPI 2.0+, use XSDT
            let extended = unsafe { &*(self.rsdp_addr as *const RsdpExtended) };
            extended.xsdt_address
        } else {
            // ACPI 1.0, use RSDT
            rsdp.rsdt_address as u64
        }
    }
}
