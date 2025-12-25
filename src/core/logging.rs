//! # Unified Logging Infrastructure
//!
//! Este módulo fornece o backend para as macros `ignite::println!` e
//! `ignite::log::*`. Ele atua como um multiplexador, enviando output para
//! múltiplos destinos (Serial, Vídeo, RAM).
//!
//! ## 🎯 Propósito e Responsabilidade
//! - **Observabilidade Precoce:** Permitir debug antes mesmo do vídeo ser
//!   inicializado (via Serial COM1).
//! - **Padronização:** Implementa a trait `log::Log`, permitindo usar o
//!   ecossistema `log` crate.
//!
//! ## 🏗️ Arquitetura
//! - **Static Global:** Usa `LOGGER` estático.
//! - **Direct Hardware Access:** Chama `arch::x86::serial` diretamente. Isos
//!   quebra camadas puras, mas é necessário no bootloader.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Simplicidade:** Não aloca memória (no-alloc), seguro para usar no panic
//!   handler.
//! - **Level Filtering:** Permite compilar builds de "Release" sem logs de
//!   "Trace" para boot mais rápido.
//!
//! ### ⚠️ Pontos de Atenção (Dívida Técnica)
//! - **Hardcoded Output:** O logger chama `crate::arch::x86::serial`
//!   diretamente. Se portarmos para ARM (UEFI usa PL011 UART), isso quebra.
//!   - *Solução:* Abstrair via `trait LogOutput`.
//! - **Output Síncrono:** A escrita na serial é bloqueante. Se o cabo serial
//!   não estiver conectado (e o hardware não tiver buffer FIFO profundo), pode
//!   atrasar o boot.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Feature)** Adicionar **In-Memory RingBuffer Logger**.
//!   - *Motivo:* Permitir descarregar logs para o Kernel (via `BootInfo`) para
//!     que o `dmesg` do Linux/Redstone mostre o que aconteceu no boot.
//! - [ ] **TODO: (Refactor)** Suportar múltiplos sinks dinâmicos (Serial + GOP
//!   + File).

use log::{LevelFilter, Log, Metadata, Record};

/// Logger global estático.
static LOGGER: GlobalLogger = GlobalLogger;

/// Trait para backends de escrita (Serial, Framebuffer).
pub trait LogWriter: Send + Sync {
    fn write_char(&mut self, c: char);
    fn write_str(&mut self, s: &str);
}

/// O Logger principal que despacha para o Writer registrado.
struct GlobalLogger;

impl Log for GlobalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Aqui conectaríamos com um SerialWriter global protegido por Spinlock.
            // Como `core` não pode depender de `hardware`, usamos uma função de hook.
            // Para simplificar este arquivo core:
            crate::arch::x86::serial::serial_print_fmt(format_args!(
                "[{}] {}\n",
                record.level(),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

/// Inicializa o sistema de logs.
pub fn init() {
    // Configura o logger global.
    // Ignoramos erro se já estiver inicializado.
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(LevelFilter::Trace);
}

// Macro helper para print sem newline (estilo print!)
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::x86::serial::serial_print_fmt(format_args!($($arg)*)));
}

// Macro helper para print com newline (estilo println!)
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
