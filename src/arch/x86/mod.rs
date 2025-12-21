//! Implementação para Arquitetura x86_64
//!
//! Contém primitivas de I/O, controle de registradores e drivers básicos
//! (Serial).

pub mod instructions;
pub mod io;
pub mod registers;
pub mod serial;

// Re-exports convenientes
pub use instructions::{hlt, pause};
pub use io::Port;
pub use registers::{flush_tlb, read_cr3, write_cr3};

/// Inicializa recursos específicos da arquitetura x86.
pub fn init() {
    // Inicializa a porta serial para logs
    serial::init_serial_early();
}
