//! Módulo Central de Gerenciamento de Memória
//!
//! Este módulo orquestra todo o subsistema de memória do bootloader, desde
//! a descoberta (UEFI Map) até a preparação final (Paging & Handoff).

// Módulos internos (privados ou públicos conforme necessidade)
pub mod allocator;
pub mod bump_allocator;
pub mod handoff;
pub mod layout;
pub mod map;
pub mod paging;
pub mod region;

// Re-exports para facilitar o uso no `main.rs`
pub use allocator::{FrameAllocator, UefiFrameAllocator};
pub use bump_allocator::BumpAllocator;
pub use handoff::BootInfo;
pub use paging::PageTableManager;

use crate::uefi::BootServices;

/// Função helper para inicializar o alocador global do Rust.
/// Deve ser chamada logo no início do `uefi_main`.
pub unsafe fn init_heap() {
    // O BumpAllocator é estático e lazy, mas se precisarmos de inicialização
    // explícita no futuro, ela vem aqui.
}

/// Helper para sair dos serviços de boot e retornar o mapa de memória.
///
/// ATENÇÃO: Após chamar isso, `print!`, `alloc!`, e UEFI morrem.
/// O controle é total do código Rust.
pub fn exit_boot_services_and_get_map(
    bs: &BootServices,
    image_handle: crate::uefi::Handle,
) -> (
    crate::uefi::table::boot::MemoryMapKey,
    impl Iterator<Item = region::PhysicalMemoryRegion>,
) {
    // Implementação encapsulada da cerimônia de ExitBootServices
    // Retorna a chave do mapa e o iterador sanitizado
    todo!("Implementar lógica de retry loop do ExitBootServices");
}
