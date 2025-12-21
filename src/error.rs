//! Sistema de erros centralizado do bootloader Ignite
//!
//! Este módulo define todos os tipos de erro que podem ocorrer durante o
//! processo de boot, organizados por categoria (FileSystem, ELF, Memory, Video,
//! etc.)

use core::fmt;

/// Tipo Result customizado para o bootloader
pub type Result<T> = core::result::Result<T, BootError>;

/// Erro principal do bootloader - engloba todos os tipos de erro possíveis
#[derive(Debug)]
pub enum BootError {
    /// Erros relacionados ao sistema de arquivos
    FileSystem(FileSystemError),
    /// Erros relacionados ao parsing/loading de ELF
    Elf(ElfError),
    /// Erros relacionados à alocação/gerenciamento de memória
    Memory(MemoryError),
    /// Erros relacionados à configuração de vídeo
    Video(VideoError),
    /// Erros relacionados à configuração
    Config(ConfigError),
    /// Erro genérico com mensagem
    Generic(&'static str),
}

/// Erros de sistema de arquivos
#[derive(Debug)]
pub enum FileSystemError {
    /// Arquivo não encontrado (nome do arquivo logado separadamente)
    FileNotFound,
    /// Erro ao ler arquivo
    ReadError,
    /// Caminho inválido
    InvalidPath,
    /// Erro ao abrir volume
    VolumeOpenError,
    /// Arquivo não é regular (é diretório, link, etc)
    NotRegularFile,
}

/// Erros de parsing/loading ELF
#[derive(Debug)]
pub enum ElfError {
    /// Erro ao parsear arquivo ELF
    ParseError,
    /// Formato ELF inválido
    InvalidFormat,
    /// Ponto de entrada inválido (0x0)
    InvalidEntryPoint,
    /// Nenhum segmento PT_LOAD encontrado
    NoLoadableSegments,
    /// Erro ao copiar segmento
    SegmentCopyError,
}

/// Erros de gerenciamento de memória
#[derive(Debug)]
pub enum MemoryError {
    /// Falha ao alocar páginas
    AllocationFailed,
    /// Endereço inválido
    InvalidAddress,
    /// Tamanho inválido
    InvalidSize,
    /// Memória insuficiente
    OutOfMemory,
}

/// Erros de configuração de vídeo
#[derive(Debug)]
pub enum VideoError {
    /// Nenhum handle GOP encontrado
    NoGopHandle,
    /// Falha ao abrir protocolo GOP
    GopOpenFailed,
    /// Modo de vídeo não suportado
    UnsupportedMode,
}

/// Erros de configuração
#[derive(Debug)]
pub enum ConfigError {
    /// Arquivo de configuração não encontrado
    NotFound,
    /// Erro ao parsear configuração
    ParseError,
    /// Configuração inválida
    Invalid(&'static str),
}

// Implementações de Display para mensagens de erro amigáveis

impl fmt::Display for BootError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BootError::FileSystem(e) => write!(f, "Erro de sistema de arquivos: {}", e),
            BootError::Elf(e) => write!(f, "Erro de ELF: {}", e),
            BootError::Memory(e) => write!(f, "Erro de memória: {}", e),
            BootError::Video(e) => write!(f, "Erro de vídeo: {}", e),
            BootError::Config(e) => write!(f, "Erro de configuração: {}", e),
            BootError::Generic(msg) => write!(f, "Erro: {}", msg),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileSystemError::FileNotFound => write!(f, "Arquivo não encontrado"),
            FileSystemError::ReadError => write!(f, "Erro ao ler arquivo"),
            FileSystemError::InvalidPath => write!(f, "Caminho inválido"),
            FileSystemError::VolumeOpenError => write!(f, "Erro ao abrir volume"),
            FileSystemError::NotRegularFile => write!(f, "Não é um arquivo regular"),
        }
    }
}

impl fmt::Display for ElfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ElfError::ParseError => write!(f, "Erro ao parsear arquivo ELF"),
            ElfError::InvalidFormat => write!(f, "Formato ELF inválido"),
            ElfError::InvalidEntryPoint => write!(f, "Ponto de entrada inválido (0x0)"),
            ElfError::NoLoadableSegments => write!(f, "Nenhum segmento PT_LOAD encontrado"),
            ElfError::SegmentCopyError => write!(f, "Erro ao copiar segmento"),
        }
    }
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::AllocationFailed => write!(f, "Falha ao alocar memória"),
            MemoryError::InvalidAddress => write!(f, "Endereço de memória inválido"),
            MemoryError::InvalidSize => write!(f, "Tamanho de memória inválido"),
            MemoryError::OutOfMemory => write!(f, "Memória insuficiente"),
        }
    }
}

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VideoError::NoGopHandle => write!(f, "Nenhum handle GOP encontrado"),
            VideoError::GopOpenFailed => write!(f, "Falha ao abrir protocolo GOP"),
            VideoError::UnsupportedMode => write!(f, "Modo de vídeo não suportado"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "Arquivo de configuração não encontrado"),
            ConfigError::ParseError => write!(f, "Erro ao parsear configuração"),
            ConfigError::Invalid(msg) => write!(f, "Configuração inválida: {}", msg),
        }
    }
}

// Conversões de erros UEFI para nossos tipos

impl From<uefi::Error> for BootError {
    fn from(_e: uefi::Error) -> Self {
        BootError::Generic("Erro UEFI")
    }
}

impl From<goblin::error::Error> for BootError {
    fn from(_: goblin::error::Error) -> Self {
        BootError::Elf(ElfError::ParseError)
    }
}
