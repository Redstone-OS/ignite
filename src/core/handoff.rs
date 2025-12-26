//! # Boot Information Handoff (ABI)
//!
//! Este arquivo define a **Interface Binária (ABI)** crítica entre Bootloader e
//! Kernel. As estruturas aqui definidas não são apenas structs Rust; são blocos
//! de memória bruta que devem obedecer a um layout rígido.
//!
//! ## ⚠️ O Contrato de Sangue (Blood Pact)
//! 1. **Layout Fixo:** Todas as structs DEVEM usar `#[repr(C)]`.
//! 2. **Tipos Primitivos:** Proibido usar `Vec`, `String`, `Option`, `Result`
//!    ou qualquer tipo com layout dinâmico/opaco.
//! 3. **Versionamento:** O campo `version` existe para prevenir que um Ignite
//!    v2 carregue um Forge v1 (e exploda tudo).
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Simplicidade:** A struct `BootInfo` é um POD (Plain Old Data) simples.
//! - **Flexibilidade:** Suporta diferentes formatos de pixel (`PixelFormat`) e
//!   tipos de memória (`MemoryType`), abstraindo x86/UEFI.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Duplicação de Código:** Este arquivo é uma cópia *manual* de
//!   `forge/src/core/handoff.rs`.
//!   - *Risco:* Se alguém editar lá e esquecer aqui, o Kernel lerá lixo e
//!     causará um **Double Fault** ou comportamento errático.
//! - **Magic Numbers:** A assinatura `BOOT_INFO_MAGIC` é boa, mas não há
//!   checksum de integridade (CRC32).
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Architecture)** Mover este arquivo para uma crate
//!   compartilhada `redstone-abi` ou `redstone-common`.
//!   - *Motivo:* Garantir "Single Source of Truth" em tempo de compilação.
//! - [ ] **TODO: (Testing)** Adicionar teste de
//!   `assert_eq!(size_of::<BootInfo>(), ...)` no CI.
//!   - *Meta:* Falhar build se o tamanho da struct mudar sem alterar a versão.
//! - [ ] **TODO: (Cleanup)** Remover structs `MemoryInfo` e `KernelInfo`
//!   marcadas como Legacy.

/// Assinatura mágica para validar que o BootInfo é legítimo ("REDSTONE" em
/// ASCII).
pub const BOOT_INFO_MAGIC: u64 = 0x524544_53544F4E45;

/// Versão atual da estrutura de BootInfo. Incrementar se mudar o layout.
/// v2: Adicionado _padding e cr3_phys para alinhamento correto com kernel.
pub const BOOT_INFO_VERSION: u32 = 2;

/// Informações completas de Boot entregues ao Kernel.
/// DEVE corresponder EXATAMENTE a forge/src/core/handoff.rs::BootInfo
///
/// # Layout ABI (Crítico!)
/// - `#[repr(C)]` garante layout previsível
/// - `_padding` alinha framebuffer em 8 bytes
/// - Todos os campos são tipos primitivos (sem Vec, String, Option)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    /// Assinatura mágica (deve ser verificada pelo Kernel).
    pub magic: u64,

    /// Versão do protocolo de boot.
    pub version: u32,

    /// Padding para alinhamento de 8 bytes (campos seguintes são u64).
    /// O kernel DEVE ter este campo também para manter ABI.
    pub _padding: u32,

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

    /// Endereço FÍSICO do CR3 (PML4) configurado pelo bootloader.
    /// O kernel herda esta hierarquia de page tables e NÃO deve liberar esses
    /// frames. IMPORTANTE: Este é o endereço físico real, não virtual!
    pub cr3_phys: u64,
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

// Nota: Structs legacy (MemoryInfo, KernelInfo) removidas na v2.
// Todos os dados agora passam via BootInfo unificado.
