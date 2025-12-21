//! Alocador Heap Estático (Bump Pointer)
//!
//! Implementa `GlobalAlloc` de forma simples e segura para ambientes
//! single-threaded.

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

/// Alocador "Bump" (Incremento Linear).
pub struct BumpAllocator {
    heap_start:  UnsafeCell<usize>,
    heap_end:    UnsafeCell<usize>,
    next:        UnsafeCell<usize>,
    allocations: UnsafeCell<usize>,
}

// SAFETY: O Bootloader UEFI roda em um único core/thread durante o boot
// services.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start:  UnsafeCell::new(0),
            heap_end:    UnsafeCell::new(0),
            next:        UnsafeCell::new(0),
            allocations: UnsafeCell::new(0),
        }
    }

    /// Inicializa o alocador com um bloco de memória.
    ///
    /// # Safety
    /// O chamador deve garantir que o intervalo de memória [heap_start,
    /// heap_start + heap_size) é válido e não está em uso.
    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        *self.heap_start.get() = heap_start;
        *self.heap_end.get() = heap_start + heap_size;
        *self.next.get() = heap_start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap_start = *self.heap_start.get();
        let heap_end = *self.heap_end.get();
        let mut next = *self.next.get();

        if heap_start == 0 {
            return null_mut(); // Não inicializado
        }

        let alloc_start = align_up(next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return null_mut(),
        };

        if alloc_end > heap_end {
            return null_mut(); // OOM
        }

        *self.next.get() = alloc_end;
        *self.allocations.get() += 1;

        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        *self.allocations.get() -= 1;
        if *self.allocations.get() == 0 {
            *self.next.get() = *self.heap_start.get();
        }
    }
}

/// Alinha o endereço para cima.
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
