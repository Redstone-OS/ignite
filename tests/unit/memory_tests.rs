//! Testes Unitários para o módulo de memória
//!
//! Testa alocação, paging e gerenciamento de memória.

#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;

/// Testa alinhamento de endereços
#[test]
fn test_address_alignment() {
    const PAGE_SIZE: u64 = 4096;

    fn is_aligned(addr: u64, align: u64) -> bool {
        addr % align == 0
    }

    assert!(is_aligned(0x1000, PAGE_SIZE));
    assert!(is_aligned(0x2000, PAGE_SIZE));
    assert!(!is_aligned(0x1001, PAGE_SIZE));
    assert!(!is_aligned(0x1FFF, PAGE_SIZE));

    // Huge pages (2MB)
    const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;
    assert!(is_aligned(0x200000, HUGE_PAGE_SIZE));
    assert!(!is_aligned(0x200001, HUGE_PAGE_SIZE));
}

/// Testa cálculo de número de páginas
#[test]
fn test_pages_calculation() {
    fn bytes_to_pages(bytes: usize) -> usize {
        (bytes + 4095) / 4096
    }

    assert_eq!(bytes_to_pages(0), 0);
    assert_eq!(bytes_to_pages(1), 1);
    assert_eq!(bytes_to_pages(4095), 1);
    assert_eq!(bytes_to_pages(4096), 1);
    assert_eq!(bytes_to_pages(4097), 2);
    assert_eq!(bytes_to_pages(8192), 2);
    assert_eq!(bytes_to_pages(8193), 3);
}

/// Testa extração de índices de página
#[test]
fn test_page_table_indices() {
    fn extract_indices(virt_addr: u64) -> (usize, usize, usize, usize, usize) {
        let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
        let offset = (virt_addr & 0xFFF) as usize;

        (pml4_idx, pdpt_idx, pd_idx, pt_idx, offset)
    }

    // Teste com endereço 0x0
    let (pml4, pdpt, pd, pt, off) = extract_indices(0x0);
    assert_eq!((pml4, pdpt, pd, pt, off), (0, 0, 0, 0, 0));

    // Teste com higher-half kernel
    let (pml4, pdpt, pd, pt, _) = extract_indices(0xFFFFFFFF80000000);
    assert_eq!(pml4, 511); // Último PML4 entry
    assert_eq!(pdpt, 510);

    // Teste com endereço não alinhado
    let (_, _, _, _, off) = extract_indices(0x1234);
    assert_eq!(off, 0x234);
}

/// Testa flags de entrada de página
#[test]
fn test_page_entry_flags() {
    const PAGE_PRESENT: u64 = 1 << 0;
    const PAGE_WRITABLE: u64 = 1 << 1;
    const PAGE_USER: u64 = 1 << 2;
    const PAGE_WRITE_THROUGH: u64 = 1 << 3;
    const PAGE_CACHE_DISABLE: u64 = 1 << 4;
    const PAGE_ACCESSED: u64 = 1 << 5;
    const PAGE_DIRTY: u64 = 1 << 6;
    const PAGE_HUGE: u64 = 1 << 7;
    const PAGE_GLOBAL: u64 = 1 << 8;
    const PAGE_NO_EXEC: u64 = 1 << 63;

    fn has_flag(entry: u64, flag: u64) -> bool {
        (entry & flag) != 0
    }

    // Kernel page (RW, NX)
    let kernel_data = PAGE_PRESENT | PAGE_WRITABLE | PAGE_NO_EXEC;
    assert!(has_flag(kernel_data, PAGE_PRESENT));
    assert!(has_flag(kernel_data, PAGE_WRITABLE));
    assert!(!has_flag(kernel_data, PAGE_USER));
    assert!(has_flag(kernel_data, PAGE_NO_EXEC));

    // Kernel code (RX)
    let kernel_code = PAGE_PRESENT;
    assert!(has_flag(kernel_code, PAGE_PRESENT));
    assert!(!has_flag(kernel_code, PAGE_WRITABLE));
    assert!(!has_flag(kernel_code, PAGE_NO_EXEC));

    // User page (URW, NX)
    let user_data = PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER | PAGE_NO_EXEC;
    assert!(has_flag(user_data, PAGE_USER));
}

/// Testa conversão de endereço físico para virtual (direct map)
#[test]
fn test_phys_to_virt_conversion() {
    const DIRECT_MAP_OFFSET: u64 = 0xFFFF800000000000;

    fn phys_to_virt(paddr: u64) -> u64 {
        DIRECT_MAP_OFFSET + paddr
    }

    fn virt_to_phys(vaddr: u64) -> Option<u64> {
        if vaddr >= DIRECT_MAP_OFFSET {
            Some(vaddr - DIRECT_MAP_OFFSET)
        } else {
            None
        }
    }

    assert_eq!(phys_to_virt(0x0), DIRECT_MAP_OFFSET);
    assert_eq!(phys_to_virt(0x1000), DIRECT_MAP_OFFSET + 0x1000);

    assert_eq!(virt_to_phys(DIRECT_MAP_OFFSET), Some(0x0));
    assert_eq!(virt_to_phys(DIRECT_MAP_OFFSET + 0x1000), Some(0x1000));
    assert_eq!(virt_to_phys(0x1000), None); // Não é direct map
}

/// Testa validação de range de memória
#[test]
fn test_memory_range_validation() {
    struct MemoryRange {
        base:   u64,
        length: u64,
    }

    impl MemoryRange {
        fn end(&self) -> u64 {
            self.base + self.length
        }

        fn overlaps(&self, other: &MemoryRange) -> bool {
            self.base < other.end() && other.base < self.end()
        }

        fn contains(&self, addr: u64) -> bool {
            addr >= self.base && addr < self.end()
        }
    }

    let range1 = MemoryRange {
        base:   0x1000,
        length: 0x1000,
    };
    let range2 = MemoryRange {
        base:   0x1500,
        length: 0x1000,
    };
    let range3 = MemoryRange {
        base:   0x3000,
        length: 0x1000,
    };

    // Overlap tests
    assert!(range1.overlaps(&range2));
    assert!(range2.overlaps(&range1));
    assert!(!range1.overlaps(&range3));

    // Contains tests
    assert!(range1.contains(0x1000));
    assert!(range1.contains(0x1FFF));
    assert!(!range1.contains(0x2000));
}

/// Testa cálculo de tamanho total de memória
#[test]
fn test_total_memory_calculation() {
    #[derive(Clone, Copy)]
    struct MemoryMapEntry {
        base:   u64,
        length: u64,
        usable: bool,
    }

    fn calculate_usable_memory(entries: &[MemoryMapEntry]) -> u64 {
        entries.iter().filter(|e| e.usable).map(|e| e.length).sum()
    }

    let entries = [
        MemoryMapEntry {
            base:   0x0,
            length: 0x100000,
            usable: true,
        },
        MemoryMapEntry {
            base:   0x100000,
            length: 0x100000,
            usable: false,
        },
        MemoryMapEntry {
            base:   0x200000,
            length: 0x200000,
            usable: true,
        },
    ];

    let total = calculate_usable_memory(&entries);
    assert_eq!(total, 0x100000 + 0x200000);
}

/// Testa bitmap allocator
#[test]
fn test_bitmap_allocator() {
    struct BitmapAllocator {
        bitmap:     Vec<u64>,
        base_frame: u64,
    }

    impl BitmapAllocator {
        fn new(num_frames: usize, base_frame: u64) -> Self {
            let num_words = (num_frames + 63) / 64;
            Self {
                bitmap: alloc::vec![0; num_words],
                base_frame,
            }
        }

        fn allocate(&mut self) -> Option<u64> {
            for (word_idx, word) in self.bitmap.iter_mut().enumerate() {
                if *word != !0 {
                    for bit_idx in 0..64 {
                        if (*word & (1 << bit_idx)) == 0 {
                            *word |= 1 << bit_idx;
                            let frame_idx = word_idx * 64 + bit_idx;
                            return Some(self.base_frame + (frame_idx as u64));
                        }
                    }
                }
            }
            None
        }

        fn free(&mut self, frame: u64) {
            let frame_idx = (frame - self.base_frame) as usize;
            let word_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;

            if word_idx < self.bitmap.len() {
                self.bitmap[word_idx] &= !(1 << bit_idx);
            }
        }
    }

    let mut allocator = BitmapAllocator::new(128, 0x1000);

    let frame1 = allocator.allocate().unwrap();
    let frame2 = allocator.allocate().unwrap();
    let frame3 = allocator.allocate().unwrap();

    assert_eq!(frame1, 0x1000);
    assert_eq!(frame2, 0x1001);
    assert_eq!(frame3, 0x1002);

    allocator.free(frame2);
    let frame4 = allocator.allocate().unwrap();
    assert_eq!(frame4, 0x1001); // Reusa frame liberado
}

/// Testa aritmética de ponteiros
#[test]
fn test_pointer_arithmetic() {
    let base: usize = 0x100000;
    let offset: isize = 0x1000;

    let ptr = base as *mut u8;
    let offset_ptr = unsafe { ptr.offset(offset) };

    assert_eq!(offset_ptr as usize, base + (offset as usize));
}

/// Testa conversão de tamanhos
#[test]
fn test_size_conversions() {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    assert_eq!(4 * KB, 4096);
    assert_eq!(2 * MB, 2 * 1024 * 1024);
    assert_eq!(1 * GB, 1 * 1024 * 1024 * 1024);
}

/// Testa validação de stack pointer
#[test]
fn test_stack_pointer_validation() {
    fn is_valid_stack(rsp: u64, stack_base: u64, stack_size: u64) -> bool {
        rsp >= stack_base && rsp < stack_base + stack_size && rsp % 16 == 0 // x86_64 requer alinhamento de 16 bytes
    }

    let stack_base = 0xFFFF_FFFF_C000_0000;
    let stack_size = 16 * 4096; // 64KB

    assert!(is_valid_stack(
        stack_base + stack_size - 16,
        stack_base,
        stack_size
    ));
    assert!(is_valid_stack(stack_base + 0x1000, stack_base, stack_size));
    assert!(!is_valid_stack(stack_base + 1, stack_base, stack_size)); // Não alinhado
    assert!(!is_valid_stack(stack_base - 16, stack_base, stack_size)); // Fora do range
}

/// Testa cálculo de fragmentação de memória
#[test]
fn test_memory_fragmentation() {
    struct Allocation {
        size: usize,
    }

    fn calculate_fragmentation(allocs: &[Allocation], total_memory: usize) -> f64 {
        let used: usize = allocs.iter().map(|a| a.size).sum();
        let free = total_memory - used;

        if total_memory == 0 {
            0.0
        } else {
            (free as f64) / (total_memory as f64)
        }
    }

    let allocs = vec![Allocation { size: 1000 }, Allocation { size: 2000 }];

    let total = 10000;
    let frag = calculate_fragmentation(&allocs, total);

    assert!((frag - 0.7).abs() < 0.01); // ~70% livre
}
