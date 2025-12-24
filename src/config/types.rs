//! Tipos de Configuração do Bootloader
//!
//! Define as estruturas de dados que representam o estado configurado do
//! sistema.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Configuração global do Bootloader.
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Tempo em segundos antes de iniciar a entrada padrão.
    pub timeout: Option<u32>,

    /// Índice da entrada padrão.
    pub default_entry_idx: usize,

    /// Se verdadeiro, suprime logs não críticos.
    pub quiet: bool,

    /// Habilita saída serial.
    pub serial_enabled: bool,

    /// Resolução desejada.
    pub resolution: Option<(u32, u32)>,

    /// Caminho do wallpaper.
    pub wallpaper: Option<String>,

    /// Lista de sistemas operacionais.
    pub entries: Vec<Entry>,
}

impl Default for BootConfig {
    /// Configuração Padrão (Limpa).
    /// Usada pelo Parser como base para ler o arquivo de configuração.
    fn default() -> Self {
        Self {
            timeout:           Some(5),
            default_entry_idx: 0,
            quiet:             false,
            serial_enabled:    true,
            resolution:        None,
            wallpaper:         None,
            entries:           Vec::new(), // IMPORTANTE: Começa vazio para não duplicar entradas
        }
    }
}

impl BootConfig {
    /// Retorna a configuração de Recuperação (Recovery).
    /// Deve ser usada APENAS quando o arquivo de configuração não for
    /// encontrado.
    pub fn recovery() -> Self {
        let recovery_entry = Entry {
            name:     "UEFI Shell (Recovery)".to_string(),
            protocol: Protocol::EfiChainload,
            path:     "boot():/EFI/BOOT/shellx64.efi".to_string(),
            cmdline:  None,
            modules:  Vec::new(),
            dtb_path: None,
        };

        // Usa os defaults, mas adiciona a entrada de rescue
        let mut config = Self::default();
        config.entries.push(recovery_entry);
        config
    }
}

/// Uma entrada no menu de boot.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name:     String,
    pub protocol: Protocol,
    pub path:     String,
    pub cmdline:  Option<String>,
    pub modules:  Vec<Module>,
    pub dtb_path: Option<String>,
}

/// Módulo carregável (InitRD, Drivers).
#[derive(Debug, Clone)]
pub struct Module {
    pub path:    String,
    pub cmdline: Option<String>,
}

/// Protocolos suportados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Linux,
    Limine,
    Redstone,
    EfiChainload,
    Multiboot2,
    Unknown,
}

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "linux" => Protocol::Linux,
            "limine" => Protocol::Limine,
            "redstone" | "native" => Protocol::Redstone,
            "efi" | "chainload" => Protocol::EfiChainload,
            "multiboot2" => Protocol::Multiboot2,
            _ => Protocol::Unknown,
        }
    }
}
