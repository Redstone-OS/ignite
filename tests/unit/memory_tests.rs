//! Testes Unitários para Memory Management
//!
//! Este módulo testa o gerenciamento de memória do bootloader,
//! incluindo o bump allocator e operações de alocação.

#![cfg(test)]

use core::ptr::NonNull;

/// Simula um bump allocator simples para testes
struct SimpleBumpAllocator {
    heap_start: usize,
    heap_end:   usize,
    next:       usize,
}

impl SimpleBumpAllocator {
    const fn new(heap_start: usize, heap_size: usize) -> Self {
        Self {
            heap_start,
            heap_end: heap_start + heap_size,
            next: heap_start,
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Alinhar next
        let aligned_addr = (self.next + align - 1) & !(align - 1);
        let alloc_end = aligned_addr.checked_add(size)?;

        if alloc_end > self.heap_end {
            return None; // Sem espaço
        }

        self.next = alloc_end;
        NonNull::new(aligned_addr as *mut u8)
    }

    fn used(&self) -> usize {
        self.next - self.heap_start
    }

    fn remaining(&self) -> usize {
        self.heap_end - self.next
    }
}

#[test]
fn test_bump_allocator_criacao() {
    // Arrange & Act: Criar allocator
    let allocator = SimpleBumpAllocator::new(0x10000, 0x1000);

    // Assert: Verificar inicialização
    assert_eq!(allocator.heap_start, 0x10000);
    assert_eq!(allocator.heap_end, 0x11000);
    assert_eq!(allocator.next, 0x10000);
    assert_eq!(allocator.used(), 0);
    assert_eq!(allocator.remaining(), 0x1000);
}

#[test]
fn test_bump_allocator_alocacao_simples() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x1000);

    // Act: Alocar 64 bytes
    let ptr = allocator.alloc(64, 1);

    // Assert: Deve alocar com sucesso
    assert!(ptr.is_some());
    assert_eq!(allocator.used(), 64);
    assert_eq!(allocator.remaining(), 0x1000 - 64);
}

#[test]
fn test_bump_allocator_alocacao_alinhada() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x1000);

    // Act: Alocar com alinhamento de 16 bytes
    let ptr1 = allocator.alloc(10, 16);

    // Assert: Endereço deve estar alinhado
    assert!(ptr1.is_some());
    let addr = ptr1.unwrap().as_ptr() as usize;
    assert_eq!(addr % 16, 0, "Endereço deve estar alinhado a 16 bytes");
}

#[test]
fn test_bump_allocator_multiplas_alocacoes() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x1000);

    // Act: Fazer múltiplas alocações
    let ptr1 = allocator.alloc(32, 1);
    let ptr2 = allocator.alloc(64, 1);
    let ptr3 = allocator.alloc(128, 1);

    // Assert: Todas devem ter sucesso
    assert!(ptr1.is_some());
    assert!(ptr2.is_some());
    assert!(ptr3.is_some());

    // Verificar que são diferentes
    assert_ne!(ptr1.unwrap().as_ptr(), ptr2.unwrap().as_ptr());
    assert_ne!(ptr2.unwrap().as_ptr(), ptr3.unwrap().as_ptr());

    // Verificar espaço usado
    assert_eq!(allocator.used(), 32 + 64 + 128);
}

#[test]
fn test_bump_allocator_sem_espaco() {
    // Arrange: Criar allocator pequeno
    let mut allocator = SimpleBumpAllocator::new(0x10000, 100);

    // Act: Tentar alocar mais que o disponível
    let ptr = allocator.alloc(200, 1);

    // Assert: Deve falhar
    assert!(ptr.is_none(), "Alocação deveria falhar quando sem espaço");
}

#[test]
fn test_bump_allocator_preencher_heap() {
    // Arrange: Criar allocator
    let heap_size = 1024usize;
    let mut allocator = SimpleBumpAllocator::new(0x10000, heap_size);

    // Act: Alocar exatamente o tamanho da heap
    let ptr = allocator.alloc(heap_size, 1);

    // Assert: Deve ter sucesso
    assert!(ptr.is_some());
    assert_eq!(allocator.used(), heap_size);
    assert_eq!(allocator.remaining(), 0);

    // Tentar alocar mais deve falhar
    let ptr2 = allocator.alloc(1, 1);
    assert!(ptr2.is_none());
}

#[test]
fn test_alinhamento_potencia_de_dois() {
    // Arrange: Testar diferentes alinhamentos
    let alinhamentos = [1, 2, 4, 8, 16, 32, 64, 128, 256];

    for align in alinhamentos {
        // Act: Criar allocator e alocar
        let mut allocator = SimpleBumpAllocator::new(0x10000, 0x10000);
        let ptr = allocator.alloc(100, align);

        // Assert: Endereço deve estar alinhado
        assert!(ptr.is_some(), "Falha ao alocar com alinhamento {}", align);
        let addr = ptr.unwrap().as_ptr() as usize;
        assert_eq!(addr % align, 0, "Endereço não alinhado para {}", align);
    }
}

#[test]
fn test_alinhamento_incrementa_next() {
    // Arrange: Criar allocator em endereço não alinhado
    let mut allocator = SimpleBumpAllocator::new(0x10001, 0x1000);

    // Act: Alocar com alinhamento de 16
    let ptr = allocator.alloc(8, 16);

    // Assert: Deve alinhar corretamente
    assert!(ptr.is_some());
    let addr = ptr.unwrap().as_ptr() as usize;
    assert_eq!(addr % 16, 0);

    // Deve ter usado mais que 8 bytes devido ao alinhamento
    assert!(allocator.used() >= 8);
}

#[test]
fn test_alocacao_zero_bytes() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x1000);

    // Act: Tentar alocar 0 bytes
    let ptr = allocator.alloc(0, 1);

    // Assert: Comportamento depende da implementação
    // Geralmente retorna um ponteiro válido mas não avança
    if let Some(_p) = ptr {
        assert_eq!(allocator.used(), 0, "Alocar 0 bytes não deve usar espaço");
    }
}

#[test]
fn test_calculo_espacoouso() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 1024);

    // Act: Fazer alocações
    allocator.alloc(100, 1);
    let usado1 = allocator.used();

    allocator.alloc(200, 1);
    let usado2 = allocator.used();

    // Assert: Uso deve aumentar
    assert!(usado2 > usado1);
    assert_eq!(usado1, 100);
    assert_eq!(usado2, 300);
}

#[test]
fn test_calculo_espaco_restante() {
    // Arrange: Criar allocator
    let heap_size = 1024usize;
    let mut allocator = SimpleBumpAllocator::new(0x10000, heap_size);

    // Act & Assert: Verificar espaço restante
    assert_eq!(allocator.remaining(), heap_size);

    allocator.alloc(100, 1);
    assert_eq!(allocator.remaining(), heap_size - 100);

    allocator.alloc(200, 1);
    assert_eq!(allocator.remaining(), heap_size - 300);
}

#[test]
fn test_alocacao_grande() {
    // Arrange: Criar allocator grande
    let heap_size = 1024 * 1024usize; // 1 MiB
    let mut allocator = SimpleBumpAllocator::new(0x100000, heap_size);

    // Act: Alocar memória grande
    let tamanho_alocacao = 512 * 1024; // 512 KiB
    let ptr = allocator.alloc(tamanho_alocacao, 1);

    // Assert: Deve ter sucesso
    assert!(ptr.is_some());
    assert_eq!(allocator.used(), tamanho_alocacao);
}

#[test]
fn test_overflow_deteccao() {
    // Arrange: Criar allocator com endereço alto
    let heap_start = usize::MAX - 1000;
    let mut allocator = SimpleBumpAllocator::new(heap_start, 500);

    // Act: Tentar alocar mais que o possível sem overflow
    let ptr = allocator.alloc(600, 1);

    // Assert: Deve falhar (sem overflow)
    assert!(ptr.is_none(), "Deveria detectar overflow");
}

#[test]
fn test_alinhamento_preservado_sequencial() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x10000);

    // Act: Fazer alocações sequenciais alinhadas
    let ptr1 = allocator.alloc(16, 16);
    let ptr2 = allocator.alloc(32, 32);
    let ptr3 = allocator.alloc(64, 64);

    // Assert: Todos devem estar corretamente alinhados
    assert!(ptr1.is_some());
    assert!(ptr2.is_some());
    assert!(ptr3.is_some());

    assert_eq!(ptr1.unwrap().as_ptr() as usize % 16, 0);
    assert_eq!(ptr2.unwrap().as_ptr() as usize % 32, 0);
    assert_eq!(ptr3.unwrap().as_ptr() as usize % 64, 0);
}

#[test]
fn test_memoria_limites() {
    // Arrange: Criar allocator
    let heap_start = 0x10000usize;
    let heap_size = 1024usize;
    let allocator = SimpleBumpAllocator::new(heap_start, heap_size);

    // Assert: Verificar limites
    assert_eq!(allocator.heap_start, heap_start);
    assert_eq!(allocator.heap_end, heap_start + heap_size);
    assert!(allocator.next >= heap_start);
    assert!(allocator.next <= heap_start + heap_size);
}

#[test]
fn test_alocacao_tamanho_maximo() {
    // Arrange: Criar allocator
    let heap_size = 4096usize;
    let mut allocator = SimpleBumpAllocator::new(0x10000, heap_size);

    // Act: Alocar exatamente o máximo disponível
    let ptr = allocator.alloc(heap_size, 1);

    // Assert: Deve ter sucesso
    assert!(ptr.is_some());
    assert_eq!(allocator.remaining(), 0);
}
