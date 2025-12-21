//! Alocador de memória para o bootloader
//!
//! Wrapper em torno dos serviços de alocação UEFI

use crate::{
    error::{BootError, MemoryError, Result},
    uefi::{
        BootServices,
        table::boot::{AllocateType, MemoryType},
    },
};

/// Alocador de memória que usa serviços UEFI
pub struct MemoryAllocator<'a> {
    boot_services: &'a BootServices,
}

impl<'a> MemoryAllocator<'a> {
    /// Cria um novo alocador de memória
    pub fn new(boot_services: &'a BootServices) -> Self {
        Self { boot_services }
    }

    /// Aloca páginas de memória
    ///
    /// # Argumentos
    /// * `pages` - Número de páginas (4KB cada) a alocar
    /// * `alloc_type` - Tipo de alocação (AnyPages, Address, MaxAddress)
    ///
    /// # Retorna
    /// Endereço físico da memória alocada
    pub fn allocate_pages(&self, pages: usize, alloc_type: AllocateType) -> Result<u64> {
        unsafe {
            self.boot_services
                .allocate_pages_helper(alloc_type, MemoryType::LoaderData, pages)
                .map_err(|_| BootError::Memory(MemoryError::AllocationFailed))
        }
    }

    /// Aloca páginas em um endereço específico
    ///
    /// # Argumentos
    /// * `address` - Endereço desejado
    /// * `pages` - Número de páginas a alocar
    pub fn allocate_at_address(&self, address: u64, pages: usize) -> Result<u64> {
        self.allocate_pages(pages, AllocateType::Address(address))
    }

    /// Aloca páginas em qualquer endereço disponível
    ///
    /// # Argumentos
    /// * `pages` - Número de páginas a alocar
    pub fn allocate_any(&self, pages: usize) -> Result<u64> {
        self.allocate_pages(pages, AllocateType::AllocateAnyPages)
    }

    /// Libera páginas de memória
    ///
    /// # Argumentos
    /// * `address` - Endereço da memória a liberar
    /// * `pages` - Número de páginas a liberar
    pub fn free_pages(&self, address: u64, pages: usize) -> Result<()> {
        unsafe {
            self.boot_services
                .free_pages_helper(address, pages)
                .map_err(|_| BootError::Memory(MemoryError::InvalidAddress))
        }
    }

    /// Calcula número de páginas necessárias para um tamanho em bytes
    ///
    /// # Argumentos
    /// * `size` - Tamanho em bytes
    ///
    /// # Retorna
    /// Número de páginas (arredondado para cima)
    pub fn pages_for_size(size: usize) -> usize {
        (size + 0xFFF) / 0x1000
    }

    /// Zera uma região de memória
    ///
    /// # Argumentos
    /// * `address` - Endereço da memória
    /// * `size` - Tamanho em bytes
    pub unsafe fn zero_memory(&self, address: u64, size: usize) {
        unsafe {
            core::ptr::write_bytes(address as *mut u8, 0, size);
        }
    }
}
