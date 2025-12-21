//! Suporte a Device Tree (FDT)
//!
//! Para sistemas ARM64 e RISC-V

/// Cabeçalho FDT
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FdtHeader {
    pub magic:             u32, // 0xd00dfeed
    pub totalsize:         u32,
    pub off_dt_struct:     u32,
    pub off_dt_strings:    u32,
    pub off_mem_rsvmap:    u32,
    pub version:           u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys:   u32,
    pub size_dt_strings:   u32,
    pub size_dt_struct:    u32,
}

const FDT_MAGIC: u32 = 0xd00dfeed;

/// Gerenciador de Device Tree
pub struct DeviceTree {
    fdt_addr: u64,
}

impl DeviceTree {
    /// Carregar device tree a partir de endereço
    pub fn from_address(addr: u64) -> Option<Self> {
        let header = unsafe { &*(addr as *const FdtHeader) };

        // Validar magic (big-endian)
        if u32::from_be(header.magic) != FDT_MAGIC {
            return None;
        }

        Some(Self { fdt_addr: addr })
    }

    /// Obter tamanho do FDT
    pub fn size(&self) -> u32 {
        let header = unsafe { &*(self.fdt_addr as *const FdtHeader) };
        u32::from_be(header.totalsize)
    }

    /// Obter FDT como slice
    pub fn as_slice(&self) -> &[u8] {
        let size = self.size() as usize;
        unsafe { core::slice::from_raw_parts(self.fdt_addr as *const u8, size) }
    }
}
