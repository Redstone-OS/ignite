//! Sistema Unificado de Erros
//!
//! Define a hierarquia de erros do bootloader.
//! Projetado para ser `no_std`, exaustivo e fácil de converter.

use core::fmt;

/// Alias conveniente para Results do bootloader.
pub type Result<T> = core::result::Result<T, BootError>;

/// O erro principal que engloba todas as falhas possíveis no Bootloader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    /// Erros originados do Firmware UEFI.
    Uefi(crate::uefi::Status),

    /// Erros de Entrada/Saída de baixo nível (Hardware/Device).
    Io(IoError),

    /// Erros lógicos do Sistema de Arquivos (FAT32, RedstoneFS).
    FileSystem(FileSystemError),

    /// Erros de Gerenciamento de Memória (Alocação, Paging).
    Memory(MemoryError),

    /// Erros de Parsing ou Loading de Executáveis (ELF, PE).
    Elf(ElfError),

    /// Erros do Subsistema de Vídeo (GOP).
    Video(VideoError),

    /// Erros de Configuração (Parser, Validação).
    Config(ConfigError),

    /// Erro genérico para casos não categorizados (Stubs, TODOs).
    Generic(&'static str),

    /// Pânico controlado ou erro fatal.
    Panic(&'static str),
}

/// Erros de I/O de Dispositivo (Hardware).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    DeviceError,
    NotReady,
    Timeout,
    InvalidParameter,
}

/// Erros de Sistema de Arquivos (Lógicos).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    FileNotFound,
    InvalidPath,
    ReadError,
    WriteError,
    SeekError,
    VolumeOpenError,
    InvalidSignature,
    UnsupportedFsType,
    InvalidSize,
    NotRegularFile,
    bufferTooSmall,
    DeviceError, // Re-mapa de IO se necessário no contexto de FS
}

/// Erros de Memória.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    AllocationFailed,
    FrameAllocationFailed,
    InvalidAlignment,
    TableUpdateFailed,
    HeapFull,
    InvalidAddress,
    InvalidSize,
    OutOfMemory,
}

/// Erros de Executáveis (ELF/Kernel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    ParseError,
    InvalidMagic,
    InvalidArchitecture,
    InvalidEndianness,
    InvalidMachine,
    InvalidEntryPoint,
    UnsupportedFileType,
    NoLoadableSegments,
    SegmentMapFailed,
    SegmentCopyError,
    InvalidFormat,
}

/// Erros de Vídeo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoError {
    InitializationFailed,
    GopNotSupported,
    ModeSetFailed,
    ResolutionMismatch,
    NoGopHandle,
    OpenProtocolFailed,
    GopOpenFailed,
    UnsupportedMode,
}

/// Erros de Configuração.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    NotFound,
    ParseError,  // Renomeado de ParseFailed para consistência
    ParseFailed, // Mantido para compatibilidade se usado
    InvalidKey,
    ValueOutOfRange,
    Invalid(&'static str), // Para mensagens customizadas
}

// --- Implementações de Conversão (Syntactic Sugar) ---

impl From<crate::uefi::Status> for BootError {
    fn from(s: crate::uefi::Status) -> Self {
        BootError::Uefi(s)
    }
}

impl From<IoError> for BootError {
    fn from(e: IoError) -> Self {
        BootError::Io(e)
    }
}

impl From<FileSystemError> for BootError {
    fn from(e: FileSystemError) -> Self {
        BootError::FileSystem(e)
    }
}

impl From<MemoryError> for BootError {
    fn from(e: MemoryError) -> Self {
        BootError::Memory(e)
    }
}

impl From<ElfError> for BootError {
    fn from(e: ElfError) -> Self {
        BootError::Elf(e)
    }
}

impl From<VideoError> for BootError {
    fn from(e: VideoError) -> Self {
        BootError::Video(e)
    }
}

impl From<ConfigError> for BootError {
    fn from(e: ConfigError) -> Self {
        BootError::Config(e)
    }
}

// --- Implementação de Display (Logs) ---

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootError::Uefi(s) => write!(f, "UEFI Error: {:?}", s),
            BootError::Io(e) => write!(f, "IO Error: {:?}", e),
            BootError::FileSystem(e) => write!(f, "FS Error: {:?}", e),
            BootError::Memory(e) => write!(f, "Memory Error: {:?}", e),
            BootError::Elf(e) => write!(f, "ELF Error: {:?}", e),
            BootError::Video(e) => write!(f, "Video Error: {:?}", e),
            BootError::Config(e) => write!(f, "Config Error: {:?}", e),
            BootError::Generic(s) => write!(f, "Generic Error: {}", s),
            BootError::Panic(s) => write!(f, "Panic: {}", s),
        }
    }
}

// Display para sub-erros (simplificado para debug)
impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for ElfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
