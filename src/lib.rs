//! # Ignite Bootloader Library
//!
//! A `ignite-lib` é a coleção de subsistemas modulares que compõem o
//! bootloader. Ela é agnóstica do ponto de entrada (`main.rs`), permitindo que
//! seja usada em testes unitários ou em diferentes targets UEFI.
//!
//! ## 🏗️ Arquitetura Modular
//! O Ignite segue uma arquitetura em camadas para isolar a complexidade do
//! firmware UEFI:
//!
//! ### 1. Camada de Abstração (Hardware/Firmware)
//! - [`uefi`]: Wrappers Rust-safe para a API C da UEFI (System Table, Boot
//!   Services).
//! - [`arch`]: Código Assembly específico para x86_64 (Port I/O, paging).
//! - [`video`]: Gerenciamento de GOP (Graphics Output Protocol).
//!
//! ### 2. Camada de Core (Lógica de Boot)
//! - [`memory`]: Alocadores (Bump Allocator) e Gerenciamento de Páginas.
//! - [`fs`]: Drivers de sistema de arquivos (abstração sobre protocolo
//!   SimpleFS).
//! - [`config`]: Parser do manifesto `ignite.cfg` (TOML-like).
//!
//! ### 3. Camada de Aplicação (UI & Security)
//! - [`ui`]: Framework de UI imediata (texto e gráficos) para o menu de boot.
//! - [`security`]: Verificação de assinaturas (Secure Boot) e TPM.
//! - [`recovery`]: Ferramentas de diagnóstico pré-boot.
//!
//! ## ⚠️ Notas de Engenharia
//! - **No Std:** Esta library não depende da `std`.
//! - **Allocation:** Depende da crate `alloc`. O binário consumidor deve
//!   fornecer um `#[global_allocator]`.
//! - **Panic:** Fornece um handler `panic_impl` que imprime na tela e serial,
//!   mas o binário deve registrá-lo.
//!
//! ## 🛠️ TODOs (Library Level)
//! - [ ] **TODO: (Refactor)** Separar `uefi` em uma crate externa ou usar
//!   `uefi-rs` puro (upstream).
//!   - *Motivo:* Manter bindings UEFI manuais é propenso a erro e redundante.
//! - [ ] **TODO: (Test)** Criar target de teste em QEMU/OVMF automatizado.

#![no_std]
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
