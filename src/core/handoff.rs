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

use core::fmt;

/// Assinatura mágica para validar que o BootInfo é legítimo ("IGNITE!" em
/// ASCII).
pub const BOOT_INFO_MAGIC: u64 = 0x2145_5449_4E47_4900;

/// Versão atual da estrutura de BootInfo. Incrementar se mudar o layout.
pub const BOOT_INFO_VERSION: u64 = 1;

/// Informações completas de Boot entregues ao Kernel.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BootInfo {
    /// Assinatura mágica (deve ser verificada pelo Kernel).
    pub magic:   u64,
    /// Versão da estrutura.
    pub version: u64,

    /// Informações sobre a memória física (RAM).
    pub memory: MemoryInfo,

    /// Informações sobre o Framebuffer de vídeo (GOP).
    pub framebuffer: FramebufferInfo,

    /// Informações sobre o Kernel carregado.
    pub kernel: KernelInfo,

    /// Endereço do InitRamdisk (se houver).
    pub initrd_addr: u64,
    /// Tamanho do InitRamdisk.
    pub initrd_size: u64,

    /// Ponteiro para a tabela ACPI RSDP (Root System Description Pointer).
    pub rsdp_addr: u64,

    /// Ponteiro para a tabela do sistema UEFI (para acesso a Runtime Services).
    /// O Kernel deve mapear isso corretamente se quiser usar variáveis NVRAM.
    pub uefi_system_table: u64,
}

/// Detalhes sobre o Framebuffer Gráfico.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Endereço físico do buffer de pixels.
    pub address: u64,
    /// Tamanho total em bytes.
    pub size:    usize,
    /// Largura em pixels.
    pub width:   u32,
    /// Altura em pixels.
    pub height:  u32,
    /// Pixels por linha (stride).
    pub stride:  u32,
    /// Formato de pixel (ver enum PixelFormat no módulo video).
    pub format:  u32,
}

/// Detalhes sobre o Kernel carregado.
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

/// Resumo do mapa de memória.
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
