//! Entrada e Saída de Portas (Port I/O)
//!
//! Abstração segura sobre as instruções `in` e `out` do Assembly x86.

use core::marker::PhantomData;

/// Representa uma porta de I/O tipada.
///
/// # Generics
/// * `T`: O tipo de dado transferido (u8, u16, u32).
#[derive(Debug, Clone, Copy)]
pub struct Port<T> {
    port:    u16,
    phantom: PhantomData<T>,
}

impl<T> Port<T> {
    /// Cria uma nova porta insegura.
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            phantom: PhantomData,
        }
    }
}

/// Implementação para bytes (u8)
impl Port<u8> {
    /// Lê um byte da porta.
    #[inline]
    pub unsafe fn read(&self) -> u8 {
        let value: u8;
        core::arch::asm!("in al, dx", out("al") value, in("dx") self.port, options(nomem, nostack, preserves_flags));
        value
    }

    /// Escreve um byte na porta.
    #[inline]
    pub unsafe fn write(&mut self, value: u8) {
        core::arch::asm!("out dx, al", in("dx") self.port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

/// Implementação para palavras (u16)
impl Port<u16> {
    #[inline]
    pub unsafe fn read(&self) -> u16 {
        let value: u16;
        core::arch::asm!("in ax, dx", out("ax") value, in("dx") self.port, options(nomem, nostack, preserves_flags));
        value
    }

    #[inline]
    pub unsafe fn write(&mut self, value: u16) {
        core::arch::asm!("out dx, ax", in("dx") self.port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

/// Helper para esperar um ciclo de I/O (usado em hardware antigo).
#[inline]
pub unsafe fn io_wait() {
    // Escreve em uma porta não utilizada (0x80) para gastar ciclos
    Port::<u8>::new(0x80).write(0);
}
