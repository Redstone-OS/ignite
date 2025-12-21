//! Tipos de Configuração
//!
//! Estruturas de dados para configuração de boot

use alloc::{string::String, vec::Vec};

/// Configuração completa de boot
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Timeout em segundos antes do auto-boot (None = sem timeout)
    pub timeout: Option<u32>,

    /// Índice da entrada padrão (1-based)
    pub default_entry: usize,

    /// Modo silencioso (suprimir saída)
    pub quiet: bool,

    /// Habilitar saída serial
    pub serial: bool,

    /// Baudrate serial (Apenas BIOS)
    pub serial_baudrate: u32,

    /// Modo verboso
    pub verbose: bool,

    /// Resolução da interface (WxH)
    pub interface_resolution: Option<(u32, u32)>,

    /// Texto de branding da interface
    pub interface_branding: Option<String>,

    /// Caminho do wallpaper
    pub wallpaper: Option<String>,

    /// Estilo do wallpaper: tiled, centered, stretched
    pub wallpaper_style: WallpaperStyle,

    /// Editor habilitado
    pub editor_enabled: bool,

    /// Entradas de menu
    pub entries: Vec<MenuEntry>,
}

impl Default for BootConfig {
    fn default() -> Self {
        Self {
            timeout:              Some(5),
            default_entry:        1,
            quiet:                false,
            serial:               false,
            serial_baudrate:      115200,
            verbose:              false,
            interface_resolution: None,
            interface_branding:   None,
            wallpaper:            None,
            wallpaper_style:      WallpaperStyle::Stretched,
            editor_enabled:       true,
            entries:              Vec::new(),
        }
    }
}

/// Entrada de menu
#[derive(Debug, Clone)]
pub struct MenuEntry {
    /// Nome/título da entrada
    pub name: String,

    /// Comentário exibido quando selecionado
    pub comment: Option<String>,

    /// Protocolo de boot: limine, linux, multiboot1, multiboot2, efi, bios
    pub protocol: String,

    /// Caminho do kernel/executável
    pub kernel_path: String,

    /// Argumentos de linha de comando
    pub cmdline: Option<String>,

    /// Módulos/initrd
    pub modules: Vec<Module>,

    /// Resolução de vídeo (WxHxBPP)
    pub resolution: Option<(u32, u32, u32)>,

    /// Modo texto (apenas BIOS)
    pub textmode: bool,

    /// Caminho do device tree blob
    pub dtb_path: Option<String>,

    /// KASLR habilitado
    pub kaslr: bool,

    /// Sub-entradas (para menus hierárquicos)
    pub sub_entries: Vec<MenuEntry>,

    /// Expandido por padrão
    pub expanded: bool,
}

impl MenuEntry {
    pub fn new(name: String, protocol: String, kernel_path: String) -> Self {
        Self {
            name,
            comment: None,
            protocol,
            kernel_path,
            cmdline: None,
            modules: Vec::new(),
            resolution: None,
            textmode: false,
            dtb_path: None,
            kaslr: false,
            sub_entries: Vec::new(),
            expanded: false,
        }
    }
}

/// Módulo (initrd, ramdisk, etc.)
#[derive(Debug, Clone)]
pub struct Module {
    /// Caminho do módulo
    pub path: String,

    /// Linha de comando / string do módulo
    pub cmdline: Option<String>,
}

/// Estilo de wallpaper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallpaperStyle {
    Tiled,
    Centered,
    Stretched,
}

impl WallpaperStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tiled" => Some(Self::Tiled),
            "centered" => Some(Self::Centered),
            "stretched" => Some(Self::Stretched),
            _ => None,
        }
    }
}
