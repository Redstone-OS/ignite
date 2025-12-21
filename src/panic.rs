//! Tratamento de Pânico (Panic Handler)
//!
//! Este arquivo define o comportamento do sistema quando ocorre um erro
//! irrecuperável (Rust panic). Em um ambiente `no_std`, somos obrigados a
//! implementar isso.
//!
//! # Comportamento Industrial
//! 1. Logar o erro na porta Serial (para captura automática em CI/QEMU).
//! 2. Imprimir na tela (para o usuário ver).
//! 3. Travar a CPU em um loop infinito (Halt) para evitar danos aos dados.

use core::panic::PanicInfo;

use crate::arch;

pub fn panic_impl(info: &PanicInfo) -> ! {
    crate::println!("\n*** FATAL SYSTEM ERROR ***");

    if let Some(location) = info.location() {
        crate::println!(
            "Local: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    // FIX: message() retorna PanicMessage diretamente em versões recentes
    // e display dele funciona. Removemos o `if let Some` incorreto.
    let msg = info.message();
    crate::println!("Erro:  {}", msg);

    crate::println!("Sistema paralisado.");
    loop {
        arch::hlt();
    }
}
