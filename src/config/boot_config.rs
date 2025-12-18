//! Configuração do bootloader
//!
//! Carrega e gerencia configurações de boot a partir de arquivo

use crate::error::Result;
use crate::fs::FileLoader;
use log::{info, warn};

/// Configuração principal do bootloader
#[derive(Debug, Clone, Copy)]
pub struct BootConfig {
    /// Configuração do menu de boot
    pub menu: BootMenuConfig,
    /// Timeout do menu em segundos (0 = sem timeout)
    pub timeout: u32,
}

impl BootConfig {
    /// Carrega configuração de arquivo
    ///
    /// TODO: Implementar parser de arquivo .ini ou .cfg
    /// Formato sugerido:
    /// ```ini
    /// [boot]
    /// menu_enabled = false
    /// default_os = redstone
    /// timeout = 5
    ///
    /// [os.redstone]
    /// name = "Redstone OS"
    /// kernel = "forge"
    /// initfs = "initfs"
    ///
    /// [os.linux]
    /// name = "Linux"
    /// kernel = "vmlinuz"
    /// initrd = "initrd.img"
    ///
    /// [os.windows]
    /// name = "Windows"
    /// efi = "\\EFI\\Microsoft\\Boot\\bootmgfw.efi"
    /// ```
    pub fn load(_file_loader: &mut FileLoader) -> Result<Self> {
        // TODO: Tentar carregar arquivo "boot.cfg" ou "ignite.ini"
        // TODO: Parsear arquivo e preencher estrutura
        // Por enquanto, retorna configuração padrão

        warn!("Carregamento de configuração não implementado - usando padrões (TODO)");
        Ok(Self::default())
    }

    /// Configuração padrão
    pub fn default() -> Self {
        Self {
            menu: BootMenuConfig {
                enabled: false, // Desabilitado por padrão
                show_on_key: true, // Pode ser ativado com tecla
            },
            timeout: 5,
        }
    }

    /// Verifica se menu deve ser exibido
    pub fn should_show_menu(&self, key_pressed: bool) -> bool {
        self.menu.enabled || (self.menu.show_on_key && key_pressed)
    }
}

/// Configuração do menu de boot
#[derive(Debug, Clone)]
pub struct BootMenuConfig {
    /// Menu habilitado por padrão
    pub enabled: bool,
    /// Permitir ativar menu com tecla
    pub show_on_key: bool,
}

/// Entrada de sistema operacional
#[derive(Debug, Clone, Copy)]
pub struct OsEntry {
    /// Nome amigável do OS
    pub name: &'static str,
    /// Tipo de OS
    pub os_type: OsType,
    /// Caminho do kernel/EFI
    pub kernel_path: &'static str,
    /// InitFS/InitRD (opcional)
    pub initfs_path: Option<&'static str>,
}

/// Tipo de sistema operacional
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    /// Redstone OS (padrão)
    Redstone,
    /// Linux genérico
    Linux,
    /// Windows (via EFI)
    Windows,
    /// Outro sistema UEFI
    Other,
}

impl OsEntry {
    /// Cria entrada para Redstone OS
    pub fn redstone() -> Self {
        Self {
            name: "Redstone OS",
            os_type: OsType::Redstone,
            kernel_path: "forge",
            initfs_path: Some("initfs"),
        }
    }

    /// Cria entrada para Linux
    ///
    /// TODO: Detectar automaticamente instalações Linux
    pub fn linux() -> Self {
        Self {
            name: "Linux",
            os_type: OsType::Linux,
            kernel_path: "vmlinuz",
            initfs_path: Some("initrd.img"),
        }
    }

    /// Cria entrada para Windows
    ///
    /// TODO: Detectar automaticamente instalação Windows
    pub fn windows() -> Self {
        Self {
            name: "Windows",
            os_type: OsType::Windows,
            kernel_path: "\\EFI\\Microsoft\\Boot\\bootmgfw.efi",
            initfs_path: None,
        }
    }
}

/// Lista de sistemas operacionais disponíveis
pub struct OsList {
    pub entries: &'static [OsEntry],
    pub count: usize,
}

impl OsList {
    /// Detecta sistemas operacionais disponíveis
    ///
    /// TODO: Implementar detecção automática
    /// - Procurar por kernels Linux em /boot
    /// - Procurar por bootmgfw.efi do Windows
    /// - Procurar por outros bootloaders UEFI
    pub fn detect(_file_loader: &mut FileLoader) -> Self {
        warn!("Detecção automática de OS não implementada (TODO)");

        // Por enquanto, retorna apenas Redstone
        // TODO: Usar alocação dinâmica ou lazy_static para lista de OS
        Self {
            entries: &[],
            count: 0,
        }
    }

    /// Encontra entrada por nome
    pub fn find(&self, name: &str) -> Option<&OsEntry> {
        self.entries.iter().find(|e| e.name == name)
    }
}
