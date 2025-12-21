//! Abstração de Arquitetura de Hardware
//!
//! Este módulo exporta interfaces uniformes para manipulação de CPU, I/O e
//! registradores específicos. O Bootloader seleciona a implementação correta
//! em tempo de compilação.

// === x86_64 (AMD64) - Alvo Principal Atual ===
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod x86;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use x86::*;

// === AArch64 (ARM64) - Placeholder Futuro ===
#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

// === RISC-V 64 - Placeholder Futuro ===
#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

/// Interface comum que todas as arquiteturas devem implementar (Traits).
/// Garante que o Core possa chamar funções de arquitetura sem saber qual é.
pub trait Architecture {
    /// Inicializa a CPU/Arquitetura.
    fn init();

    /// Pausa a CPU (Halt) até a próxima interrupção.
    fn hlt();
}
