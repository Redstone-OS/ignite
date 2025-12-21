use core::marker::PhantomData;

/// Trait de IO para leitura/escrita genérica
pub trait Io {
    type Value: Copy + PartialEq + core::fmt::Debug;
    fn read(&self) -> Self::Value;
    fn write(&mut self, value: Self::Value);
}

/// Wrapper Apenas-Leitura
pub struct ReadOnly<T> {
    inner: T,
}

impl<T> ReadOnly<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Io> Io for ReadOnly<T> {
    type Value = T::Value;

    fn read(&self) -> T::Value {
        self.inner.read()
    }

    fn write(&mut self, _value: T::Value) {
        // No-op ou panic
        // No-op ou pânico
    }
}

/// Port I/O para x86
#[derive(Clone, Copy)]
pub struct Pio<T> {
    port:     u16,
    _phantom: PhantomData<T>,
}

impl<T> Pio<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _phantom: PhantomData,
        }
    }
}

impl Io for Pio<u8> {
    type Value = u8;

    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            core::arch::asm!("in al, dx", out("al") value, in("dx") self.port, options(nomem, nostack, preserves_flags));
        }
        value
    }

    fn write(&mut self, value: u8) {
        unsafe {
            core::arch::asm!("out dx, al", in("al") value, in("dx") self.port, options(nomem, nostack, preserves_flags));
        }
    }
}

// Implementação mínima de MMIO se necessário
pub struct Mmio<T> {
    addr:     usize,
    _phantom: PhantomData<T>,
}

impl<T> Mmio<T> {
    pub unsafe fn new(addr: usize) -> Self {
        Self {
            addr,
            _phantom: PhantomData,
        }
    }
}

impl Io for Mmio<u32> {
    type Value = u32;
    fn read(&self) -> u32 {
        unsafe { core::ptr::read_volatile(self.addr as *const u32) }
    }
    fn write(&mut self, value: u32) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u32, value) }
    }
}

impl Io for Mmio<u8> {
    type Value = u8;
    fn read(&self) -> u8 {
        unsafe { core::ptr::read_volatile(self.addr as *const u8) }
    }
    fn write(&mut self, value: u8) {
        unsafe { core::ptr::write_volatile(self.addr as *mut u8, value) }
    }
}
