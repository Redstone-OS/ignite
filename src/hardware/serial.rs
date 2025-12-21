//! Driver Serial UART 16550
//!
//! Usa as primitivas de arquitetura (`Port<T>`) para comunicação.
//! Implementa `fmt::Write` para integração com o sistema de logs.

use core::fmt;

use crate::arch::io::Port; // Usa a abstração do módulo arch

const COM1_BASE: u16 = 0x3F8;

pub struct SerialPort {
    data:       Port<u8>,
    int_en:     Port<u8>,
    fifo_ctrl:  Port<u8>,
    line_ctrl:  Port<u8>,
    modem_ctrl: Port<u8>,
    line_sts:   Port<u8>,
}

impl SerialPort {
    /// Cria uma interface para a porta COM1 padrão.
    pub const fn new() -> Self {
        Self {
            data:       Port::new(COM1_BASE),
            int_en:     Port::new(COM1_BASE + 1),
            fifo_ctrl:  Port::new(COM1_BASE + 2),
            line_ctrl:  Port::new(COM1_BASE + 3),
            modem_ctrl: Port::new(COM1_BASE + 4),
            line_sts:   Port::new(COM1_BASE + 5),
        }
    }

    /// Inicializa o UART com configurações padrão (115200 baud, 8N1).
    pub fn init(&mut self) {
        unsafe {
            // Desabilitar interrupções
            self.int_en.write(0x00);

            // Habilitar DLAB (para configurar baud rate)
            self.line_ctrl.write(0x80);

            // Configurar divisor para 115200 baud (divisor = 1)
            self.data.write(0x01); // Low byte
            self.int_en.write(0x00); // High byte

            // 8 bits, sem paridade, 1 stop bit
            self.line_ctrl.write(0x03);

            // Habilitar FIFO, limpar buffers, trigger level 14 bytes
            self.fifo_ctrl.write(0xC7);

            // Habilitar IRQs, RTS/DSR
            self.modem_ctrl.write(0x0B);
        }
    }

    /// Verifica se a linha de transmissão está vazia.
    fn is_transmit_empty(&self) -> bool {
        unsafe { (self.line_sts.read() & 0x20) != 0 }
    }

    /// Envia um byte. Bloqueia até que o buffer esteja livre.
    pub fn send(&mut self, byte: u8) {
        while !self.is_transmit_empty() {
            crate::arch::pause(); // Hint para a CPU
        }
        unsafe {
            self.data.write(byte);
        }
    }

    /// Envia uma string.
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Traduz newline para CRLF (padrão serial)
                b'\n' => {
                    self.send(b'\r');
                    self.send(b'\n');
                },
                b => self.send(b),
            }
        }
    }
}

// Integração com macro write! e sistema de logs
impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}
