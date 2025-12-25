//! Boot Information Handoff (ABI)
//!
//! Este arquivo define as estruturas de dados que são passadas do Bootloader
//! para o Kernel. É o contrato de dados (Data Contract).
//!
//! # Regras de Ouro (Nível Industrial)
//! 1. Tudo deve ser `#[repr(C)]` para garantir layout de memória consistente.
//! 2. Versionamento é obrigatório (`version` field) para evitar
//!    incompatibilidades.
//! 3. Sem tipos complexos do Rust (Vec, String). Apenas primitivos e ponteiros.
//!
//! IMPORTANTE: Esta estrutura DEVE estar 100% sincronizada com
//! forge/src/core/handoff.rs


/// Assinatura mágica para validar que o BootInfo é legítimo ("REDSTONE" em
/// ASCII).
pub const BOOT_INFO_MAGIC: u64 = 0x524544_53544F4E45;

/// Versão atual da estrutura de BootInfo. Incrementar se mudar o layout.
pub const BOOT_INFO_VERSION: u32 = 1;

/// Informações completas de Boot entregues ao Kernel.
/// DEVE corresponder EXATAMENTE a forge/src/core/handoff.rs::BootInfo
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Assinatura mágica (deve ser verificada pelo Kernel).
    pub magic: u64,

    /// Versão do protocolo de boot.
    pub version: u32,

    /// Informações de vídeo (GOP).
    pub framebuffer: FramebufferInfo,

    /// Mapa de memória física.
    pub memory_map_addr: u64,
    pub memory_map_len:  u64,

    /// Tabela ACPI RSDP (Root System Description Pointer).
    pub rsdp_addr: u64,

    /// Localização física do Kernel.
    pub kernel_phys_addr: u64,
    pub kernel_size:      u64,

    /// Endereço do Initramfs (se carregado).
    pub initramfs_addr: u64,
    pub initramfs_size: u64,
}

/// Detalhes sobre o Framebuffer Gráfico.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Endereço físico do buffer de pixels.
    pub addr:   u64,
    /// Tamanho total em bytes.
    pub size:   u64,
    /// Largura em pixels.
    pub width:  u32,
    /// Altura em pixels.
    pub height: u32,
    /// Pixels por linha (stride).
    pub stride: u32,
    /// Formato de pixel (como u32 para compatibilidade C).
    pub format: PixelFormat,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb = 0,
    Bgr = 1,
    Bitmask = 2,
    BltOnly = 3,
}

/// Entrada do mapa de memória física
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub len:  u64,
    pub typ:  MemoryType,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Usable = 1,
    Reserved = 2,
    AcpiReclaimable = 3,
    AcpiNvs = 4,
    BadMemory = 5,
    BootloaderReclaimable = 6,
    KernelAndModules = 7,
    Framebuffer = 8,
}

// =============================================================================
// Estruturas antigas (deprecated - mantidas para compatibilidade temporária)
// =============================================================================

/// Resumo do mapa de memória (LEGACY - não usar)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Ponteiro para o array de regiões de memória.
    pub map_addr:       u64,
    /// Número de entradas no mapa.
    pub map_count:      usize,
    /// Endereço físico da Tabela de Páginas (PML4/CR3) ativa.
    pub page_table_cr3: u64,
}

/// Detalhes sobre o Kernel carregado (LEGACY - não usar)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KernelInfo {
    /// Endereço físico onde o kernel foi carregado.
    pub phys_addr:   u64,
    /// Endereço virtual de entrada (Entry Point).
    pub entry_point: u64,
    /// Tamanho total em memória.
    pub size:        u64,
    /// Base da Stack inicial configurada pelo bootloader.
    pub stack_base:  u64,
    /// Tamanho da Stack.
    pub stack_size:  u64,
}
