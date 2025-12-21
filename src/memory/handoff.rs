//! Estrutura de Boot Information (Handoff)
//!
//! Dados passados para o Kernel.

use crate::memory::layout::BootLayout; // Import absoluto para evitar erros de 'super'
use crate::memory::region::PhysicalMemoryRegion;

/// Estrutura principal de troca de dados (ABI estável).
#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    /// Versão da estrutura.
    pub version: u64,

    /// Mapa de memória sanitizado.
    pub memory_map: &'static [PhysicalMemoryRegion],

    /// Endereço físico da PML4 ativa.
    pub page_table_cr3: u64,

    /// Endereço virtual onde o Kernel foi mapeado.
    pub kernel_virt_base: u64,

    /// Endereço físico onde o Kernel foi carregado.
    pub kernel_phys_base: u64,

    /// Tamanho do Kernel em bytes.
    pub kernel_size: u64,

    /// Layout de memória configurado pelo bootloader.
    pub layout: BootLayout,
}

impl BootInfo {
    pub const MAGIC: u64 = 0x4947_4E49_5445_4F53; // "IGNITEOS"
}
