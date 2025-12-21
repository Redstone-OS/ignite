//! Alocador Heap Estático (Bump Pointer)
//!
//! Implementa `GlobalAlloc`. É simples, determinístico e não fragmenta.
//! Ideal para bootloaders que alocam muito e liberam tudo de uma vez
//! (quando passam o controle pro kernel).

use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr::null_mut,
};

use super::layout::BOOTLOADER_HEAP_SIZE;

/// O símbolo `HEAP` reside na seção `.bss`.
/// Garantimos alinhamento de 16 bytes para SIMD/SSE básico se necessário.
#[repr(C, align(16))]
struct HeapStorage([u8; BOOTLOADER_HEAP_SIZE]);

static mut HEAP_MEMORY: HeapStorage = HeapStorage([0; BOOTLOADER_HEAP_SIZE]);

/// Alocador "Bump" (Incremento Linear).
///
/// Funciona como uma pilha: `next` aponta para o próximo byte livre.
/// Alocação é O(1) (só soma ponteiro). Desalocação é "no-op" (vazamento
/// intencional).
pub struct BumpAllocator {
    next: UnsafeCell<usize>,
}

// SAFETY: Bootloader é single-threaded neste estágio (UEFI main thread).
// Se introduzirmos APs (Application Processors), precisaremos de um Mutex real.
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            next: UnsafeCell::new(0),
        }
    }

    /// Reinicia o alocador (perigoso, usar apenas em panic ou reset).
    pub unsafe fn reset(&self) {
        *self.next.get() = 0;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap_start = &mut HEAP_MEMORY.0 as *mut u8 as usize;
        let heap_end = heap_start + BOOTLOADER_HEAP_SIZE;

        let next_ptr = self.next.get();
        let mut current_offset = *next_ptr;
        let current_addr = heap_start + current_offset;

        // Alinhamento: move o ponteiro para frente até satisfazer `layout.align()`
        let align_offset = match current_addr % layout.align() {
            0 => 0,
            remainder => layout.align() - remainder,
        };

        // Overflow check (segurança industrial)
        let new_offset = match current_offset.checked_add(align_offset + layout.size()) {
            Some(o) => o,
            None => return null_mut(), // Overflow aritmético
        };

        if heap_start + new_offset > heap_end {
            return null_mut(); // Out of memory (OOM)
        }

        // Commit da alocação
        *next_ptr = new_offset;
        (current_addr + align_offset) as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator não libera memória individualmente.
        // A memória é recuperada em massa quando o Kernel sobrescreve a região
        // do Bootloader.
    }
}
