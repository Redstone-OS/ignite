//! Static Bump Allocator - Heap Estática em .bss
//!
//! Usa buffer estático que existe desde o início e sobrevive ao
//! exit_boot_services().

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

/// Tamanho da heap estática (4MB)
pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

/// Heap estática - alocada na .bss section
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Bump Allocator
pub struct BumpAllocator {
    next: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            next: UnsafeCell::new(0),
        }
    }

    /// Inicializar - deve ser chamado ANTES de init()
    pub unsafe fn init(&self) {
        unsafe {
            *self.next.get() = core::ptr::addr_of_mut!(HEAP).cast::<u8>() as usize;
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let heap_start = core::ptr::addr_of_mut!(HEAP).cast::<u8>() as usize;
            let heap_end = heap_start + HEAP_SIZE;

            let next = *self.next.get();

            // Auto-init se necessário
            if next == 0 {
                *self.next.get() = heap_start;
                return self.alloc(layout);
            }

            let alloc_start = align_up(next, layout.align());
            let alloc_end = match alloc_start.checked_add(layout.size()) {
                Some(end) => end,
                None => return null_mut(),
            };

            if alloc_end > heap_end {
                return null_mut();
            }

            *self.next.get() = alloc_end;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator não desaloca
    }
}

#[inline]
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
