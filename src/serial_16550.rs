use core::{
    convert::TryInto,
    fmt,
    ptr::{addr_of, addr_of_mut},
};

use bitflags::bitflags;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::io::Pio;
use crate::io::{Io, Mmio, ReadOnly};

bitflags! {
    /// Flags de habilitação de interrupção
    struct IntEnFlags: u8 {
        const RECEIVED = 1;
        const SENT = 1 << 1;
        const ERRORED = 1 << 2;
        const STATUS_CHANGE = 1 << 3;
        // 4 a 7 não usados
    }
}

bitflags! {
    /// Flags de status da linha
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 a 4 desconhecidos
        const OUTPUT_EMPTY = 1 << 5;
        // 6 e 7 desconhecidos
    }
}

#[allow(dead_code)]
#[repr(C, packed)]
pub struct SerialPort<T: Io> {
    /// Registrador de dados, leitura para receber, escrita para enviar
    data:       T,
    /// Habilitar interrupção
    int_en:     T,
    /// Controle FIFO
    fifo_ctrl:  T,
    /// Controle de linha
    line_ctrl:  T,
    /// Controle de modem
    modem_ctrl: T,
    /// Status da linha
    line_sts:   ReadOnly<T>,
    /// Status do modem
    modem_sts:  ReadOnly<T>,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SerialPort<Pio<u8>> {
    pub const fn new(base: u16) -> SerialPort<Pio<u8>> {
        SerialPort {
            data:       Pio::new(base),
            int_en:     Pio::new(base + 1),
            fifo_ctrl:  Pio::new(base + 2),
            line_ctrl:  Pio::new(base + 3),
            modem_ctrl: Pio::new(base + 4),
            line_sts:   ReadOnly::new(Pio::new(base + 5)),
            modem_sts:  ReadOnly::new(Pio::new(base + 6)),
        }
    }
}

impl SerialPort<Mmio<u32>> {
    pub unsafe fn new(base: usize) -> &'static mut SerialPort<Mmio<u32>> {
        unsafe { &mut *(base as *mut Self) }
    }
}

impl<T: Io> SerialPort<T>
where
    T::Value: From<u8> + TryInto<u8>,
{
    pub fn init(&mut self) {
        unsafe {
            // TODO: Limpeza
            // CORRIGIR: Corrigir UB se desalinhado
            (*addr_of_mut!(self.int_en)).write(0x00.into());
            (*addr_of_mut!(self.line_ctrl)).write(0x80.into());
            (*addr_of_mut!(self.data)).write(0x01.into());
            (*addr_of_mut!(self.int_en)).write(0x00.into());
            (*addr_of_mut!(self.line_ctrl)).write(0x03.into());
            (*addr_of_mut!(self.fifo_ctrl)).write(0xC7.into());
            (*addr_of_mut!(self.modem_ctrl)).write(0x0B.into());
            (*addr_of_mut!(self.int_en)).write(0x01.into());
        }
    }

    fn line_sts(&self) -> LineStsFlags {
        LineStsFlags::from_bits_truncate(
            ((unsafe { &*addr_of!(self.line_sts) }.read())
                .try_into()
                .unwrap_or(0))
                & 0xFF,
        )
    }

    pub fn receive(&mut self) -> Option<u8> {
        if self.line_sts().contains(LineStsFlags::INPUT_FULL) {
            Some(
                ((unsafe { &*addr_of!(self.data) }.read())
                    .try_into()
                    .unwrap_or(0))
                    & 0xFF,
            )
        } else {
            None
        }
    }

    pub fn send(&mut self, data: u8) {
        while !self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY) {}
        unsafe { &mut *addr_of_mut!(self.data) }.write(data.into())
    }

    pub fn write(&mut self, buf: &[u8]) {
        for &b in buf {
            match b {
                8 | 0x7F => {
                    self.send(8);
                    self.send(b' ');
                    self.send(8);
                },
                b'\n' => {
                    self.send(b'\r');
                    self.send(b'\n');
                },
                _ => {
                    self.send(b);
                },
            }
        }
    }
}

impl<T: Io> fmt::Write for SerialPort<T>
where
    T::Value: From<u8> + TryInto<u8>,
{
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write(s.as_bytes());
        Ok(())
    }
}
