//! Abstração de I/O Mapeado em Memória (MMIO)
//!
//! Fornece wrappers seguros para leitura e escrita volátil em endereços
//! físicos. Diferente do Port I/O (que fica em `arch`), MMIO é um conceito
//! genérico.

use core::marker::PhantomData;

/// Representa um registrador mapeado em memória.
#[repr(transparent)]
pub struct Mmio<T> {
    addr:     usize,
    _phantom: PhantomData<T>,
}

impl<T> Mmio<T> {
    /// Cria um novo acesso MMIO a partir de um endereço.
    ///
    /// # Safety
    /// O chamador deve garantir que o endereço é válido e mapeado.
    pub const unsafe fn new(addr: usize) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }
}

impl Mmio<u8> {
    #[inline(always)]
    pub fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.addr as *const u8) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: u8) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u8, value) }
    }
}

impl Mmio<u16> {
    #[inline(always)]
    pub fn read(&self) -> u16 {
        unsafe { core::ptr::read_volatile(self.addr as *const u16) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: u16) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u16, value) }
    }
}

impl Mmio<u32> {
    #[inline(always)]
    pub fn read(&self) -> u32 {
        unsafe { core::ptr::read_volatile(self.addr as *const u32) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: u32) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u32, value) }
    }
}

impl Mmio<u64> {
    #[inline(always)]
    pub fn read(&self) -> u64 {
        unsafe { core::ptr::read_volatile(self.addr as *const u64) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: u64) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u64, value) }
    }
}
