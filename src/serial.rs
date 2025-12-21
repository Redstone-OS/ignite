//! Wrapper configurável para debug serial
//!
//! Permite habilitar/desabilitar output serial baseado na configuração,
//! evitando overhead em sistemas sem porta serial.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::{constants::serial::COM1_PORT, io::Pio, serial_16550::SerialPort};

/// Flag global que controla se serial está ativado
static SERIAL_ENABLED: AtomicBool = AtomicBool::new(true); // Ligado por padrão

/// Porta serial global (COM1)
static mut SERIAL_PORT: Option<SerialPort<Pio<u8>>> = None;

/// Inicializa a porta serial se habilitada
pub fn init(enabled: bool) {
    SERIAL_ENABLED.store(enabled, Ordering::Relaxed);

    if enabled {
        unsafe {
            let mut port = SerialPort::new(COM1_PORT);
            port.init();
            SERIAL_PORT = Some(port);
        }
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
    SERIAL_ENABLED.load(Ordering::Relaxed)
}

/// Envia uma string para serial (apenas se habilitado)
pub fn write_str(s: &str) {
    if !is_enabled() {
        return;
    }

    unsafe {
        if let Some(port) = &mut SERIAL_PORT {
            port.write(s.as_bytes());
        }
    }
}

/// Envia bytes para serial (apenas se habilitado)
pub fn write_bytes(bytes: &[u8]) {
    if !is_enabled() {
        return;
    }

    unsafe {
        if let Some(port) = &mut SERIAL_PORT {
            port.write(bytes);
        }
    }
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
    core::arch::asm!(
        "out dx, al",
        in("dx") COM1_PORT,
        in("al") byte,
        options(nostack, preserves_flags)
    );
}
