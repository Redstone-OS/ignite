//! Gerenciador de Alocação de Frames Físicos
//!
//! Define a interface (trait) para alocação e implementa a versão UEFI.
//!
//! # Diferença entre Allocator e BumpAllocator
//! - `Allocator` (este arquivo): Lida com PÁGINAS FÍSICAS (4KiB). Essencial
//!   para criar PageTables.
//! - `BumpAllocator`: Lida com BYTES (Heap). Essencial para `Vec`, `Box`.

use crate::{
    core::error::{BootError, MemoryError, Result},
    uefi::{
        BootServices,
        table::boot::{AllocateType, MemoryType},
    },
};

/// Trait genérico para alocadores de frames.
/// Permite trocar a implementação UEFI por uma implementação própria
/// após `ExitBootServices`.
pub trait FrameAllocator {
    /// Tenta alocar `count` páginas contíguas.
    fn allocate_frame(&mut self, count: usize) -> Result<u64>;

    /// Tenta alocar páginas num endereço específico (se possível).
    fn allocate_at(&mut self, addr: u64, count: usize) -> Result<u64>;
}

/// Implementação que delega para o Firmware UEFI.
/// Só funciona ANTES de `ExitBootServices`.
pub struct UefiFrameAllocator<'a> {
    boot_services: &'a BootServices,
}

impl<'a> UefiFrameAllocator<'a> {
    pub fn new(boot_services: &'a BootServices) -> Self {
        Self { boot_services }
    }
}

impl<'a> FrameAllocator for UefiFrameAllocator<'a> {
    fn allocate_frame(&mut self, count: usize) -> Result<u64> {
        // AllocateAnyPages: O firmware escolhe o endereço (geralmente alto).
        // Usamos LoaderData para marcar que isso pertence ao nosso processo de boot.
        self.boot_services
            .allocate_pages_helper(
                AllocateType::AllocateAnyPages,
                MemoryType::LoaderData,
                count,
            )
            .map_err(|_| BootError::Memory(MemoryError::AllocationFailed))
    }

    fn allocate_at(&mut self, addr: u64, count: usize) -> Result<u64> {
        self.boot_services
            .allocate_pages_helper(
                AllocateType::AllocateAddress(addr),
                MemoryType::LoaderData,
                count,
            )
            .map_err(|_| BootError::Memory(MemoryError::AllocationFailed))
    }
}
