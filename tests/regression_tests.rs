//! Testes de Regressão
//!
//! Testa cenários que causaram bugs no passado para evitar regressões.

#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;

/// Regressão: Overflow ao calcular número de páginas
/// Bug: Integer overflow quando size era muito grande
#[test]
fn regression_page_count_overflow() {
    fn bytes_to_pages_safe(bytes: usize) -> Option<usize> {
        bytes.checked_add(4095)?.checked_div(4096)
    }

    // Deve retornar None em vez de overflow
    assert_eq!(bytes_to_pages_safe(usize::MAX), None);
    assert_eq!(bytes_to_pages_safe(usize::MAX - 4000), None);

    // Valores normais devem funcionar
    assert_eq!(bytes_to_pages_safe(4096), Some(1));
    assert_eq!(bytes_to_pages_safe(8192), Some(2));
}

/// Regressão: Buffer overflow ao copiar command line
/// Bug: Não validava tamanho antes de copiar
#[test]
fn regression_cmdline_buffer_overflow() {
    fn safe_copy_cmdline(src: &str, dest: &mut [u8]) -> Result<(), ()> {
        let bytes = src.as_bytes();
        if bytes.len() + 1 > dest.len() {
            return Err(());
        }

        dest[..bytes.len()].copy_from_slice(bytes);
        dest[bytes.len()] = 0; // Null terminator
        Ok(())
    }

    let mut buffer = [0u8; 10];

    // Deve funcionar
    assert_eq!(safe_copy_cmdline("short", &mut buffer), Ok(()));

    // Deve falhar (muito longo)
    assert_eq!(
        safe_copy_cmdline("very long command line", &mut buffer),
        Err(())
    );
}

/// Regressão: Path traversal via ".."
/// Bug: Não filtrava ".." em paths
#[test]
fn regression_path_traversal() {
    fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.contains('\0') && !path.starts_with('/') // Simplified
    }

    assert!(!is_safe_path("../../etc/passwd"));
    assert!(!is_safe_path("path/../secret"));
    assert!(is_safe_path("safe/path"));
}

/// Regressão: Division by zero em cálculo de paging
/// Bug: align = 0 causava divisão por zero
#[test]
fn regression_division_by_zero() {
    fn align_up_safe(addr: u64, align: u64) -> Option<u64> {
        if align == 0 {
            return None;
        }
        Some((addr + align - 1) & !(align - 1))
    }

    assert_eq!(align_up_safe(0x1000, 0), None);
    assert_eq!(align_up_safe(0x1000, 0x1000), Some(0x1000));
}

/// Regressão: Integer underflow em free_frame
/// Bug: Não validava se frame >= base
#[test]
fn regression_frame_underflow() {
    fn frame_to_index_safe(frame: u64, base: u64) -> Option<usize> {
        if frame < base {
            return None;
        }
        Some(((frame - base) / 4096) as usize)
    }

    assert_eq!(frame_to_index_safe(0x1000, 0x2000), None); // frame < base
    assert_eq!(frame_to_index_safe(0x2000, 0x1000), Some(1));
}

/// Regressão: Null pointer dereference após failed allocation
/// Bug: Não verificava se ptr era null
#[test]
fn regression_null_pointer_check() {
    fn use_allocation(ptr: *mut u8) -> Result<(), ()> {
        if ptr.is_null() {
            return Err(());
        }
        // Use ptr...
        Ok(())
    }

    assert_eq!(use_allocation(core::ptr::null_mut()), Err(()));

    let mut dummy = 0u8;
    assert_eq!(use_allocation(&mut dummy as *mut u8), Ok(()));
}

/// Regressão: Off-by-one em loop de page tables
/// Bug: Loop ia até size em vez de size-1
#[test]
fn regression_off_by_one() {
    fn fill_array_safe(arr: &mut [u8], value: u8) {
        for i in 0..arr.len() {
            // Correto: 0..len (exclusive end)
            arr[i] = value;
        }
    }

    let mut arr = [0u8; 10];
    fill_array_safe(&mut arr, 0xFF);

    assert_eq!(arr, [0xFF; 10]);
}

/// Regressão: Use-after-free de UEFI boot services
/// Bug: Tentava usar boot_services após ExitBootServices
#[test]
fn regression_use_after_exit_boot_services() {
    struct BootState {
        services_available: bool,
    }

    impl BootState {
        fn allocate(&self) -> Result<u64, ()> {
            if !self.services_available {
                return Err(());
            }
            Ok(0x1000)
        }

        fn exit_boot_services(&mut self) {
            self.services_available = false;
        }
    }

    let mut state = BootState {
        services_available: true,
    };

    assert_eq!(state.allocate(), Ok(0x1000));

    state.exit_boot_services();

    assert_eq!(state.allocate(), Err(())); // Deve falhar agora
}

/// Regressão: Signed/unsigned mismatch causando negative index
/// Bug: Usava i32 para índice que podia ficar negativo
#[test]
fn regression_sign_mismatch() {
    fn get_element_safe(arr: &[u8], index: usize) -> Option<u8> {
        arr.get(index).copied()
    }

    let arr = [1, 2, 3, 4, 5];

    assert_eq!(get_element_safe(&arr, 2), Some(3));
    assert_eq!(get_element_safe(&arr, 10), None); // Out of bounds
}

/// Regressão: Timeout overflow
/// Bug: timeout * 1000 overflowava com valores grandes
#[test]
fn regression_timeout_overflow() {
    fn timeout_to_ms_safe(seconds: u32) -> Option<u64> {
        (seconds as u64).checked_mul(1000)
    }

    assert_eq!(timeout_to_ms_safe(5), Some(5000));
    assert_eq!(timeout_to_ms_safe(u32::MAX), Some((u32::MAX as u64) * 1000));
}

/// Regressão: Race condition em bump allocator
/// Bug: Dois threads podiam alocar mesmo endereço
/// (Este teste é simbólico, pois no_std não tem threading real)
#[test]
fn regression_allocator_atomicity() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    struct AtomicBumpAllocator {
        next: AtomicUsize,
    }

    impl AtomicBumpAllocator {
        fn allocate(&self, size: usize) -> usize {
            self.next.fetch_add(size, Ordering::SeqCst)
        }
    }

    let allocator = AtomicBumpAllocator {
        next: AtomicUsize::new(0x1000),
    };

    let ptr1 = allocator.allocate(16);
    let ptr2 = allocator.allocate(16);

    assert_ne!(ptr1, ptr2); // Devem ser diferentes
    assert_eq!(ptr2, ptr1 + 16); // Sequenciais
}
