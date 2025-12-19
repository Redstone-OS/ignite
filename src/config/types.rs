//! Configuration Types
//!
//! Data structures for boot configuration

use alloc::{string::String, vec::Vec};

/// Complete boot configuration
#[derive(Debug, Clone)]
pub struct BootConfig {
    /// Timeout in seconds before auto-boot (None = no timeout)
    pub timeout: Option<u32>,

    /// Default entry index (1-based)
    pub default_entry: usize,

    /// Quiet mode (suppress output)
    pub quiet: bool,

    /// Enable serial output
    pub serial: bool,

    /// Serial baudrate (BIOS only)
    pub serial_baudrate: u32,

    /// Verbose mode
    pub verbose: bool,

    /// Interface resolution (WxH)
    pub interface_resolution: Option<(u32, u32)>,

    /// Interface branding text
    pub interface_branding: Option<String>,

    /// Wallpaper path
    pub wallpaper: Option<String>,

    /// Wallpaper style: tiled, centered, stretched
    pub wallpaper_style: WallpaperStyle,

    /// Editor enabled
    pub editor_enabled: bool,

    /// Menu entries
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

/// Menu entry
#[derive(Debug, Clone)]
pub struct MenuEntry {
    /// Entry name/title
    pub name: String,

    /// Comment displayed when selected
    pub comment: Option<String>,

    /// Boot protocol: limine, linux, multiboot1, multiboot2, efi, bios
    pub protocol: String,

    /// Kernel/executable path
    pub kernel_path: String,

    /// Command line arguments
    pub cmdline: Option<String>,

    /// Modules/initrd
    pub modules: Vec<Module>,

    /// Video resolution (WxHxBPP)
    pub resolution: Option<(u32, u32, u32)>,

    /// Text mode (BIOS only)
    pub textmode: bool,

    /// Device tree blob path
    pub dtb_path: Option<String>,

    /// KASLR enabled
    pub kaslr: bool,

    /// Sub-entries (for hierarchical menus)
    pub sub_entries: Vec<MenuEntry>,

    /// Expanded by default
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

/// Module (initrd, ramdisk, etc.)
#[derive(Debug, Clone)]
pub struct Module {
    /// Module path
    pub path: String,

    /// Module command line / string
    pub cmdline: Option<String>,
}

/// Wallpaper style
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
