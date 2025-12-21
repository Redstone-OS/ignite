//! Abstração de Protocolo de Boot
//!
//! Este módulo define traços e tipos para suportar múltiplos protocolos de
//! boot. Protocolos atualmente suportados:
//! - Protocolo Limine (nativo)
//! - Protocolo de Boot Linux
//! - Multiboot 2
//! - Chainloading EFI/BIOS

use crate::{error::Result, types::LoadedFile};

pub mod chainload;
pub mod limine;
pub mod linux;
pub mod multiboot2;

/// Traço de protocolo de boot
///
/// Todos os protocolos de boot devem implementar este traço para integrar com o
/// bootloader.
pub trait BootProtocol {
    /// Valida que o kernel/executável é compatível com este protocolo
    fn validate(&self, kernel: &[u8]) -> Result<()>;

    /// Prepara informações de boot e kernel para transferência (handoff)
    ///
    /// Este método deve:
    /// - Analisar cabeçalhos do kernel
    /// - Carregar segmentos do kernel na memória
    /// - Preparar informações de boot específicas do protocolo
    /// - Configurar quaisquer estruturas de dados necessárias
    fn prepare(
        &mut self,
        kernel: &[u8],
        cmdline: Option<&str>,
        modules: &[LoadedFile],
    ) -> Result<BootInfo>;

    /// Obter o endereço do ponto de entrada
    fn entry_point(&self) -> u64;

    /// Nome do protocolo para logging
    fn name(&self) -> &'static str;
}

/// Informações de boot preparadas por um protocolo
#[derive(Debug)]
pub struct BootInfo {
    /// Endereço do ponto de entrada para pular
    pub entry_point: u64,

    /// Endereço base do kernel na memória
    pub kernel_base: u64,

    /// Tamanho do kernel em bytes
    pub kernel_size: u64,

    /// Ponteiro de pilha (se o protocolo gerenciar a pilha)
    pub stack_ptr: Option<u64>,

    /// Ponteiro de informações de boot específicas do protocolo
    /// Isso será passado para o kernel em RDI (convenção de chamada x86_64)
    pub boot_info_ptr: u64,

    /// Registradores adicionais para definir (específico do protocolo)
    pub registers: ProtocolRegisters,
}

/// Valores de registrador específicos do protocolo
#[derive(Debug, Default)]
pub struct ProtocolRegisters {
    pub rax: Option<u64>,
    pub rbx: Option<u64>,
    pub rcx: Option<u64>,
    pub rdx: Option<u64>,
    pub rsi: Option<u64>,
    pub r8:  Option<u64>,
    pub r9:  Option<u64>,
}

/// Enumeração de tipos de protocolo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// Protocolo Limine nativo
    Limine,
    /// Protocolo de boot Linux
    Linux,
    /// Multiboot versão 2
    Multiboot2,
    /// Chainloading EFI
    EfiChainload,
    /// Chainloading BIOS
    BiosChainload,
}

impl ProtocolType {
    /// Analisar tipo de protocolo a partir de string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "limine" | "native" => Some(Self::Limine),
            "linux" => Some(Self::Linux),
            "multiboot2" => Some(Self::Multiboot2),
            "efi" | "uefi" | "efi_chainload" => Some(Self::EfiChainload),
            "bios" | "bios_chainload" => Some(Self::BiosChainload),
            _ => None,
        }
    }

    /// Obter nome do protocolo
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Limine => "limine",
            Self::Linux => "linux",
            Self::Multiboot2 => "multiboot2",
            Self::EfiChainload => "efi_chainload",
            Self::BiosChainload => "bios_chainload",
        }
    }
}
