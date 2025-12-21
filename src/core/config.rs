//! Configurações e Constantes Globais
//!
//! Centraliza "Magic Numbers", endereços padrão e configurações de build.
//! Facilita o tuning do bootloader sem caçar números espalhados.

/// Identidade do Bootloader.
pub mod meta {
    pub const NAME: &str = "Ignite Bootloader";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const VENDOR: &str = "Redstone OS Project";
}

/// Configurações de Memória e Stack.
pub mod memory {
    /// Tamanho da Stack que alocamos para o Kernel (64 KiB).
    pub const KERNEL_STACK_SIZE: u64 = 64 * 1024;

    /// Tamanho do Heap do Bootloader (4 MiB).
    /// Suficiente para tabelas de páginas e alocação de estruturas de arquivos.
    pub const BOOTLOADER_HEAP_SIZE: usize = 4 * 1024 * 1024;

    /// Endereço virtual onde o Kernel será linkado (Higher Half).
    /// -2GiB (0xFFFFFFFF80000000) é padrão comum em x86_64 (mcmodel=kernel).
    pub const KERNEL_VIRTUAL_BASE: u64 = 0xFFFF_8000_0000_0000;
}

/// Caminhos e Arquivos Padrão.
pub mod paths {
    pub const KERNEL_FILENAME: &str = "redstone.elf";
    pub const CONFIG_FILENAME: &str = "ignite.cfg";
    pub const LOG_FILENAME: &str = "boot.log";
}

/// Configurações de Hardware.
pub mod hardware {
    /// Porta Serial para Debug (COM1).
    pub const SERIAL_PORT_BASE: u16 = 0x3F8;
}

/// Limites de Segurança.
pub mod limits {
    /// Tamanho máximo do arquivo de config (16 KiB).
    pub const MAX_CONFIG_SIZE: usize = 16 * 1024;
    /// Tamanho máximo do Kernel (proteção contra OOM no bootloader).
    pub const MAX_KERNEL_SIZE: usize = 64 * 1024 * 1024; // 64 MB
}
