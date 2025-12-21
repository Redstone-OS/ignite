//! Estrutura de Boot Information (Handoff)
//!
//! Esta é a "caixa" que entregamos ao Kernel na função `_start`.
//! Deve ser puramente dados (POD), sem referências ao ambiente UEFI que já
//! morreu.

use super::{layout::BootLayout, region::PhysicalMemoryRegion};

/// Estrutura principal de troca de dados.
/// `#[repr(C)]` é OBRIGATÓRIO para garantir que o compilador do Kernel
/// leia os bytes na mesma ordem que o compilador do Bootloader escreveu.
#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {
    /// Versão da estrutura (para verificação de compatibilidade).
    pub version: u64,

    /// Mapa de memória sanitizado e pronto para o Kernel.
    /// O slice aponta para uma região segura copiada pelo bootloader.
    pub memory_map: &'static [PhysicalMemoryRegion],

    /// Endereço físico da PML4 (Tabela de Páginas) ativa.
    /// O Kernel precisa disso para manipular sua própria memória virtual.
    pub page_table_cr3: u64,

    /// Endereço virtual onde o Kernel foi mapeado.
    pub kernel_virt_base: u64,

    /// Endereço físico onde o Kernel foi carregado.
    pub kernel_phys_base: u64,

    /// Tamanho do Kernel em bytes.
    pub kernel_size: u64,
}

impl BootInfo {
    /// Assinatura mágica para verificar se não estamos lendo lixo.
    pub const MAGIC: u64 = 0x4947_4E49_5445_4F53; // "IGNITEOS" em hex
}
