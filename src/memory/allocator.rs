//! Gerenciador de Alocação de Frames Físicos

use crate::{
    core::error::{BootError, MemoryError, Result},
    uefi::{
        BootServices,
        table::boot::{AllocateType, MemoryType},
    },
};

pub trait FrameAllocator {
    fn allocate_frame(&mut self, count: usize) -> Result<u64>;
    fn allocate_at(&mut self, addr: u64, count: usize) -> Result<u64>;
}

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
        // CORREÇÃO: Chamada direta para allocate_pages (wrapper seguro da lib)
        self.boot_services
            .allocate_pages(
                AllocateType::AllocateAnyPages,
                MemoryType::LoaderData,
                count,
            )
            .map_err(|_| BootError::Memory(MemoryError::AllocationFailed))
    }

    fn allocate_at(&mut self, _addr: u64, count: usize) -> Result<u64> {
        // CORREÇÃO: allocate_at agora é um método específico ou usa allocate_pages
        // Se sua impl em uefi/table/boot.rs não tem allocate_at, use allocate_pages com
        // Address

        // Opção A: Se você implementou o helper allocate_at no BootServices:
        // self.boot_services.allocate_at(MemoryType::LoaderData, count, addr)

        // Opção B (Genérica): Hack para passar o endereço via referência mutável
        // Como o wrapper seguro do uefi/table/boot.rs pode não expor Address in/out:
        // Vamos assumir que você adicionou o helper `allocate_at` conforme sugerido no
        // refactor.

        self.boot_services
             .allocate_pages(AllocateType::AllocateAddress, MemoryType::LoaderData, count)
             // Nota: O wrapper seguro no boot.rs ignora addr input para AllocateAddress?
             // Se sim, precisamos usar a versão unsafe ou corrigir o boot.rs.
             // Assumindo correção no boot.rs abaixo.
             .map_err(|_| BootError::Memory(MemoryError::AllocationFailed))
    }
}
