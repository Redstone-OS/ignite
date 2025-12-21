use log::{LevelFilter, Log, Metadata, Record};

// Logger Simples
//
/// Implementação básica de logging para o bootloader.

pub static LOGGER: Logger = Logger;

/// Estrutura do Logger
pub struct Logger;

impl Logger {
    pub fn init(&'static self) {
        let _ = log::set_logger(self);
        log::set_max_level(LevelFilter::Info);
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // println!("{} - {}", record.level(), record.args());
            // Logger desabilitado temporariamente após remoção de uefi_services
        }
    }

    fn flush(&self) {}
}
