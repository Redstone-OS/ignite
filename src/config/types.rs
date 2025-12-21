//! Tipos de Configuração do Bootloader
//!
//! Define as estruturas de dados que representam o estado configurado do
//! sistema.

use alloc::{string::String, vec::Vec};

/// Configuração global do Bootloader.
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Tempo em segundos antes de iniciar a entrada padrão (None = esperar para
    /// sempre).
    pub timeout: Option<u32>,

    /// Índice da entrada padrão (0-based internamente, mas config pode ser
    /// 1-based).
    pub default_entry_idx: usize,

    /// Se verdadeiro, não imprime mensagens de log na tela (exceto erros
    /// críticos).
    pub quiet: bool,

    /// Se verdadeiro, envia logs para a porta serial.
    pub serial_enabled: bool,

    /// Configuração de resolução de vídeo (Largura, Altura).
    pub resolution: Option<(u32, u32)>,

    /// Caminho para imagem de fundo.
    pub wallpaper: Option<String>,

    /// Lista de entradas de sistemas operacionais.
    pub entries: Vec<Entry>,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            timeout:           Some(5),
            default_entry_idx: 0,
            quiet:             false,
            serial_enabled:    true,
            resolution:        None,
            wallpaper:         None,
            entries:           Vec::new(),
        }
    }
}

/// Uma entrada no menu de boot (OS ou Ferramenta).
#[derive(Debug, Clone)]
pub struct Entry {
    /// Nome exibido no menu.
    pub name: String,

    /// Protocolo de boot a ser usado.
    pub protocol: Protocol,

    /// Caminho para o executável (Kernel, EFI, etc).
    /// Suporta sintaxe de recurso: `boot(1):/kernel`.
    pub path: String,

    /// Argumentos de linha de comando para o kernel.
    pub cmdline: Option<String>,

    /// Módulos adicionais (InitRD, Drivers, etc).
    pub modules: Vec<Module>,

    /// Caminho para Device Tree Blob (ARM/RISC-V) ou sobreposição.
    pub dtb_path: Option<String>,
}

/// Módulo carregável (ex: Initramfs).
#[derive(Debug, Clone)]
pub struct Module {
    pub path:    String,
    pub cmdline: Option<String>,
}

/// Protocolos de Boot suportados.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Protocolo Linux Boot (Carrega bzImage/vmlinuz + Initrd).
    Linux,
    /// Protocolo Limine (Moderno, flexível).
    Limine,
    /// Chainload de outro executável EFI (ex: Windows Boot Manager).
    EfiChainload,
    /// Multiboot 2 (Compatibilidade legado).
    Multiboot2,
    /// Desconhecido/Inválido.
    Unknown,
}

impl From<&str> for Protocol {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "linux" => Protocol::Linux,
            "limine" => Protocol::Limine,
            "efi" | "chainload" => Protocol::EfiChainload,
            "multiboot2" => Protocol::Multiboot2,
            _ => Protocol::Unknown,
        }
    }
}
