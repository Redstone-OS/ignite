//! Driver de Porta Serial (UART 16550)
//!
//! Usado para logging via COM1 antes mesmo de termos vídeo.
//! É extremamente robusto e simples.

use core::fmt;

use super::io::Port;

// Endereços de porta padrão
const COM1: u16 = 0x3F8;

/// Inicializa a porta serial COM1 para 38400 baud.
///
/// # Safety
/// Acessa portas de I/O diretamente.
pub fn init_serial_early() {
    unsafe {
        // Desabilitar interrupções
        Port::<u8>::new(COM1 + 1).write(0x00);

        // Ativar DLAB (Divisor Latch Access Bit) para setar baud rate
        Port::<u8>::new(COM1 + 3).write(0x80);

        // Setar divisor para 38400 baud (115200 / 3) -> 3
        Port::<u8>::new(COM1 + 0).write(0x03); // Low byte
        Port::<u8>::new(COM1 + 1).write(0x00); // High byte

        // 8 bits, sem paridade, 1 stop bit
        Port::<u8>::new(COM1 + 3).write(0x03);

        // Habilitar FIFO, limpar buffers, trigger level 14 bytes
        Port::<u8>::new(COM1 + 2).write(0xC7);

        // Habilitar IRQs (RTS/DSR setados)
        Port::<u8>::new(COM1 + 4).write(0x0B);
    }
}

/// Escreve um byte na serial.
pub fn send(byte: u8) {
    unsafe {
        let status_port = Port::<u8>::new(COM1 + 5);
        let data_port = Port::<u8>::new(COM1);

        // Esperar buffer de transmissão esvaziar
        while (status_port.read() & 0x20) == 0 {
            core::hint::spin_loop();
        }

        data_port.write(byte);
    }
}

/// Escreve uma string na serial.
pub fn serial_print(s: &str) {
    for byte in s.bytes() {
        send(byte);
    }
}

/// Função helper para macros de formatação (print!).
pub fn serial_print_fmt(args: fmt::Arguments) {
    use core::fmt::Write;

    // Wrapper local para implementar fmt::Write
    struct SerialWriter;
    impl fmt::Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            serial_print(s);
            Ok(())
        }
    }

    let _ = SerialWriter.write_fmt(args);
}
