//! Wrapper configurável para debug serial
//!
//! Permite habilitar/desabilitar output serial baseado na configuração,
//! evitando overhead em sistemas sem porta serial.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::core::constants::serial::COM1_PORT;

/// Flag global que controla se serial está ativado
static SERIAL_ENABLED: AtomicBool = AtomicBool::new(true); // Ligado por padrão
static SERIAL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Inicializa a porta serial se habilitada
pub fn init(enabled: bool) {
    SERIAL_ENABLED.store(enabled, Ordering::Relaxed);

    if enabled {
        unsafe {
            // Disable interrupts
            outb(COM1_PORT + 1, 0x00);
            // Enable DLAB
            outb(COM1_PORT + 3, 0x80);
            // Set divisor (115200 baud)
            outb(COM1_PORT + 0, 0x01);
            outb(COM1_PORT + 1, 0x00);
            // 8 bits, no parity, one stop bit
            outb(COM1_PORT + 3, 0x03);
            // Enable FIFO
            outb(COM1_PORT + 2, 0xC7);
            // Enable IRQs, set RTS/DSR
            outb(COM1_PORT + 4, 0x0B);
        }
        SERIAL_INITIALIZED.store(true, Ordering::Relaxed);
    }
}

/// Habilita output serial
pub fn enable() {
    SERIAL_ENABLED.store(true, Ordering::Relaxed);
}

/// Desabilita output serial
pub fn disable() {
    SERIAL_ENABLED.store(false, Ordering::Relaxed);
}

/// Verifica se serial está habilitado
pub fn is_enabled() -> bool {
    SERIAL_ENABLED.load(Ordering::Relaxed) && SERIAL_INITIALIZED.load(Ordering::Relaxed)
}

/// Envia uma string para serial (apenas se habilitado)
pub fn write_str(s: &str) {
    if !is_enabled() {
        return;
    }

    for byte in s.bytes() {
        unsafe {
            write_byte(byte);
        }
    }
}

/// Envia bytes para serial (apenas se habilitado)
pub fn write_bytes(bytes: &[u8]) {
    if !is_enabled() {
        return;
    }

    for &byte in bytes {
        unsafe {
            write_byte(byte);
        }
    }
}

/// Escreve um byte na porta serial
unsafe fn write_byte(byte: u8) {
    // Wait for transmit buffer to be empty
    while (inb(COM1_PORT + 5) & 0x20) == 0 {}
    outb(COM1_PORT, byte);
}

/// Macro para output serial condicional
/// Uso: serial_out!("mensagem");
#[macro_export]
macro_rules! serial_out {
    ($($arg:tt)*) => {{
        if $crate::serial::is_enabled() {
            $crate::serial::write_str(&alloc::format!($($arg)*));
        }
    }};
}

/// Macro para output serial com nova linha
#[macro_export]
macro_rules! serial_outln {
    ($($arg:tt)*) => {{
        if $crate::serial::is_enabled() {
            $crate::serial::write_str(&alloc::format!($($arg)*));
            $crate::serial::write_str("\r\n");
        }
    }};
}

/// Envia um byte diretamente para porta (útil para debug muito cedo no boot)
/// Não verifica se está habilitado - use com cuidado!
#[inline]
pub unsafe fn write_byte_raw(byte: u8) {
    outb(COM1_PORT, byte);
}

/// Output byte to I/O port
#[inline]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nostack, preserves_flags)
    );
}

/// Input byte from I/O port
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") val,
        in("dx") port,
        options(nostack, preserves_flags)
    );
    val
}
