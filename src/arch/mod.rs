//! Módulos específicos de arquitetura
//!
//! Este módulo exporta definições de paginação e memória para diferentes
//! arquiteturas (x86_64, AArch64, RISC-V 64).

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;

#[cfg(target_arch = "riscv64")]
mod riscv64;
