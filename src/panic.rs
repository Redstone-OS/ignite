//! # Bootloader Panic Handler
//!
//! Este arquivo define o comportamento do sistema quando ocorre um erro
//! irrecuperável (Rust panic) durante o estágio de boot.
//!
//! ## 🎯 Comportamento de Falha (Fail-Stop)
//! Diferente do Kernel (que pode tentar matar o processo), o Bootloader não tem
//! para onde correr. Se falhar, o sistema parou.
//!
//! 1. **Diagnóstico:** Imprime o local (Arquivo:Linha) e a mensagem de erro.
//! 2. **Logging:** Envia para Serial (COM1) para captura remota.
//! 3. **Halt:** Trava a CPU (`hlt` loop) para preservar o estado da tela/logs.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Minimalismo:** Não tenta fazer limpeza complexa (unwinding), o que
//!   poderia causar *Double Panic*.
//! - **Legibilidade:** Formatação clara da mensagem de erro para o usuário
//!   final.
//!
//! ### ⚠️ Pontos de Atenção (UX & Debug)
//! - **"Tijolo" Mode:** Atualmente o handler entra em loop infinito. O usuário
//!   precisa desligar o PC no botão.
//!   - *Correção:* Deveria esperar uma tecla e reiniciar (Reboot).
//! - **Sem Backtrace:** Em erros complexos, apenas a linha do panic não é
//!   suficiente.
//!   - *Dificuldade:* Implementar stack unwinding em `no_std` é complexo e
//!     pesado para um bootloader.
//! - **Dependência de Logger:** Se o panic ocorrer *antes* da inicialização da
//!   Serial/Vídeo, nada será exibido.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (UX)** Implementar "Pressione qualquer tecla para reiniciar".
//!   - *Motivo:* Melhor experiência para o usuário em caso de falha de boot
//!     (ex: config corrompida).
//! - [ ] **TODO: (Debug)** Dump dos registradores (RAX, RBX, RIP) no panic.
//!   - *Como:* Ler o estado da CPU (se possível via inline assembly) e
//!     imprimir.

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
