//! Sistema Unificado de Erros
//!
//! Define o enum `BootError` que encapsula todas as falhas possíveis.
//! Projetado para ser `no_std` e fornecer contexto suficiente para debug.

use core::fmt;

/// Alias conveniente para Results do bootloader.
pub type Result<T> = core::result::Result<T, BootError>;

/// Categoria de falha no processo de boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    /// Erro vindo do Firmware UEFI (Status Code).
    Uefi(crate::uefi::Status),

    /// Erro de Entrada/Saída (Disco, Arquivo não encontrado).
    Io(IoError),

    /// Erro de Memória (OOM, Alinhamento, Paging).
    Memory(MemoryError),

    /// Erro no formato do Kernel (ELF inválido, Arch errada).
    Elf(ElfError),

    /// Erro de Vídeo (GOP não suportado, Resolução inválida).
    Video(VideoError),

    /// Erro de Configuração (Parse, Valor inválido).
    Config(ConfigError),

    /// Erro genérico ou pânico controlado.
    Panic(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    FileNotFound,
    DeviceError,
    BufferTooSmall,
    InvalidPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    AllocationFailed,
    FrameAllocationFailed,
    InvalidAlignment,
    TableUpdateFailed,
    HeapFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    InvalidMagic,
    InvalidArchitecture,
    InvalidEndianness,
    ParseError,
    SegmentMapFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoError {
    GopNotSupported,
    ModeSetFailed,
    ResolutionMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    ParseFailed,
    InvalidKey,
    ValueOutOfRange,
}

// Conversões Automáticas (Syntactic Sugar para uso com `?`)

impl From<crate::uefi::Status> for BootError {
    fn from(s: crate::uefi::Status) -> Self {
        BootError::Uefi(s)
    }
}

// Implementação de Display para logs amigáveis
impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::Uefi(s) => write!(f, "UEFI Error: {:?}", s),
            BootError::Io(e) => write!(f, "IO Error: {:?}", e),
            BootError::Memory(e) => write!(f, "Memory Error: {:?}", e),
            BootError::Elf(e) => write!(f, "ELF Error: {:?}", e),
            BootError::Video(e) => write!(f, "Video Error: {:?}", e),
            BootError::Config(e) => write!(f, "Config Error: {:?}", e),
            BootError::Panic(msg) => write!(f, "Critical Failure: {}", msg),
        }
    }
}
