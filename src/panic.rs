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

/// Função chamada pelo compilador em caso de pânico.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Desabilita interrupções se possível (embora em UEFI estejamos em polling)
    // unsafe { arch::x86::instructions::disable_interrupts(); }

    crate::println!("");
    crate::println!("**************************************************");
    crate::println!("               FATAL SYSTEM ERROR                 ");
    crate::println!("**************************************************");

    // Tenta extrair a localização do erro (Arquivo e Linha)
    if let Some(location) = info.location() {
        crate::println!(
            "Local: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        crate::println!("Local: Desconhecido");
    }

    // Tenta extrair a mensagem de erro
    if let Some(message) = info.message() {
        crate::println!("Erro:  {}", message);
    } else {
        crate::println!("Erro:  Causa desconhecida");
    }

    crate::println!("**************************************************");
    crate::println!("O sistema foi paralisado para evitar corrupcao.");
    crate::println!("Por favor, reinicie manualmente.");

    // Loop infinito para segurar a CPU
    loop {
        arch::hlt();
    }
}
