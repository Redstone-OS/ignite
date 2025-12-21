//! Constantes do Bootloader Ignite
//!
//! Este módulo centraliza todas as constantes usadas no bootloader,
//! evitando "magic numbers" espalhados pelo código e facilitando
//! manutenção e configuração.

/// Portas Serial (COM)
pub mod serial {
    /// Porta serial COM1 (padrão para debug)
    pub const COM1_PORT: u16 = 0x3F8;
    /// Porta serial COM2
    pub const COM2_PORT: u16 = 0x2F8;
    /// Porta serial COM3
    pub const COM3_PORT: u16 = 0x3E8;
    /// Porta serial COM4
    pub const COM4_PORT: u16 = 0x2E8;

    /// Baud rate padrão (115200)
    pub const DEFAULT_BAUD_RATE: u32 = 115200;
}

/// Paths padrão de boot
pub mod paths {
    /// Caminho padrão para o kernel
    pub const DEFAULT_KERNEL_PATH: &str = "boot/kernel";

    /// Caminho padrão para o initramfs
    pub const DEFAULT_INITFS_PATH: &str = "boot/initfs";

    /// Caminho padrão para arquivo de configuração
    pub const DEFAULT_CONFIG_PATH: &str = "ignite.conf";

    /// Caminho alternativo para configuração
    pub const ALT_CONFIG_PATH: &str = "boot.cfg";
}

/// Configurações de memória
pub mod memory {
    /// Tamanho do heap estático (4 MB)
    pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

    /// Número máximo de regiões de memória no mapa
    pub const MAX_MEMORY_REGIONS: usize = 256;

    /// Tamanho de uma página (4 KB)
    pub const PAGE_SIZE: usize = 4096;

    /// Alinhamento de stack (16 bytes para x86_64)
    pub const STACK_ALIGNMENT: u64 = 16;
}

/// Configurações de boot
pub mod boot {
    /// Delay em segundos antes de saltar para o kernel (para poder pressionar
    /// M)
    pub const BOOT_DELAY_SECONDS: u64 = 3;

    /// Tecla para ativar menu interativo
    pub const MENU_TRIGGER_KEY: char = 'M';

    /// Timeout padrão do menu (em segundos)
    pub const DEFAULT_TIMEOUT: u64 = 5;
}

/// Configurações de arquivo
pub mod file {
    /// Tamanho do chunk para leitura de arquivos grandes (1 MB)
    pub const CHUNK_SIZE: usize = 1024 * 1024;

    /// Tamanho máximo para carregar arquivo completo na memória (10 MB)
    pub const MAX_DIRECT_LOAD_SIZE: usize = 10 * 1024 * 1024;
}

/// Magic numbers e identificadores
pub mod magic {
    /// Magic number do ELF (0x7F 'E' 'L' 'F')
    pub const ELF_MAGIC: u32 = 0x464C457F;

    /// Magic number do Multiboot 1
    pub const MULTIBOOT1_MAGIC: u32 = 0x1BADB002;

    /// Magic number do Multiboot 2
    pub const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
}

/// Configurações de UI
pub mod ui {
    /// Cor de fundo do menu (RGB)
    pub const MENU_BG_COLOR: u32 = 0x1E1E2E;

    /// Cor de texto do menu (RGB)
    pub const MENU_FG_COLOR: u32 = 0xCDD6F4;

    /// Cor de seleção (RGB)
    pub const MENU_SELECTION_COLOR: u32 = 0x89B4FA;

    /// Altura da linha do menu (pixels)
    pub const MENU_LINE_HEIGHT: usize = 24;

    /// Padding do menu (pixels)
    pub const MENU_PADDING: usize = 20;
}

/// Versão do bootloader
pub const VERSION: &str = "0.4.0";

/// Nome do bootloader
pub const NAME: &str = "Ignite";

/// Banner de apresentação
pub const BANNER: &str = r#"
═══════════════════════════════════════════════════
  Ignite v0.4.0 - Bootloader UEFI
  Redstone OS
═══════════════════════════════════════════════════
"#;
