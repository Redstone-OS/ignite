//! Infraestrutura de Logging
//!
//! Permite o registro de eventos via Serial (COM1) e/ou Vídeo.
//! Utiliza a crate `log` do ecossistema Rust para padronização.

use core::fmt::Write;

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

