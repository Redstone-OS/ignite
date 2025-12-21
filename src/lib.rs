//! Ignite Bootloader Library
//!
//! Biblioteca central que fornece todos os subsistemas necessários para o
//! bootloader: UEFI, Memória, Sistema de Arquivos, UI, Segurança e Protocolos
//! de Boot.
//!
//! Esta biblioteca é `no_std` e projetada para ser consumida pelo binário
//! `main.rs`.

#![no_std]
#![feature(abi_efiapi)]
// Habilita recursos experimentais necessários para certas operações de baixo nível
#![feature(alloc_error_handler)]

extern crate alloc;

// ============================================================================
// Módulos do Sistema
// ============================================================================

// Arquitetura e Hardware
pub mod arch;
pub mod hardware;
pub mod os; // Abstração de OS para o Arch

// Core e Utilitários
pub mod config;
pub mod core;
pub mod memory;

// Formatos e Sistemas de Arquivos
pub mod elf;
pub mod fs;

// Firmware e Interfaces
pub mod uefi;
pub mod ui;
pub mod video;

// Boot e Segurança
pub mod protos;
pub mod recovery;
pub mod security;

// Tratamento de Erros Críticos
pub mod panic;

// ============================================================================
// Re-exportações (Fachada)
// ============================================================================

// Exporta tipos comuns para facilitar o uso no binário principal
pub use crate::core::{
    error::{BootError, Result},
    handoff::BootInfo,
    logging,
};

// ============================================================================
// Alocador Global (Feature Opcional)
// ============================================================================

// Permite que a biblioteca forneça o alocador se o binário não quiser
// implementar o seu próprio. No nosso caso, o main.rs geralmente define o seu,
// mas deixamos isso preparado para testes.
#[cfg(feature = "lib_allocator")]
#[global_allocator]
static ALLOCATOR: memory::BumpAllocator = memory::BumpAllocator::new();

// ============================================================================
// Helpers Globais
// ============================================================================

/// Helper para o binário chamar em caso de pânico.
pub fn panic_handler_impl(info: &::core::panic::PanicInfo) -> ! {
    crate::panic::panic_impl(info)
}
