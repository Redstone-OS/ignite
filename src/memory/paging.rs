//! Gerenciamento de Tabelas de Página (x86_64 Paging)
//!
//! Responsável por criar o espaço de endereçamento virtual inicial para o
//! Kernel. Cria mapeamentos 4-level (PML4) com suporte a Huge Pages (2MiB) para
//! eficiência.

use super::allocator::FrameAllocator;
use crate::core::error::{BootError, MemoryError, Result};

// Constantes de flags x86_64
const PAGE_PRESENT: u64 = 1 << 0;
const PAGE_WRITABLE: u64 = 1 << 1;
const PAGE_HUGE: u64 = 1 << 7; // Em PD/PDPT significa página de 2MiB/1GiB
const PAGE_NO_EXEC: u64 = 1 << 63; // NX Bit (Segurança)

/// Gerenciador de Tabelas de Página.
/// Mantém o endereço físico da PML4 raiz.
pub struct PageTableManager {
    pml4_phys_addr: u64,
}

impl PageTableManager {
    /// Cria uma nova tabela PML4 limpa e mapeia a si mesma recursivamente
    /// (opcional) ou prepara para uso.
    pub fn new(allocator: &mut impl FrameAllocator) -> Result<Self> {
        // 1. Aloca um frame para ser a PML4 raiz
        let pml4 = allocator.allocate_frame(1)?;

        // 2. Zera a tabela (segurança: não herdar lixo da RAM)
        unsafe {
            let ptr = pml4 as *mut u64;
            // 512 entradas * 8 bytes = 4096 bytes
            core::ptr::write_bytes(ptr, 0, 512);
        }

        Ok(Self {
            pml4_phys_addr: pml4,
        })
    }

    /// Retorna o endereço físico da PML4 (para carregar no registrador CR3).
    pub fn pml4_addr(&self) -> u64 {
        self.pml4_phys_addr
    }

    /// Cria um mapeamento Identity (Virtual == Físico).
    /// CRITICO: Essencial para que o bootloader continue funcionando após
    /// trocar CR3! Sem isso, o código do bootloader causa page fault
    /// imediato.
    pub fn identity_map(
        &mut self,
        phys_addr: u64,
        count: usize,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // Identity map: virtual = physical
        for i in 0..count {
            let addr = phys_addr + (i as u64 * 4096);
            self.map_page(addr, addr, PAGE_PRESENT | PAGE_WRITABLE, allocator)?;
        }
        Ok(())
    }

    /// Mapeia o Kernel no Higher Half (ou onde o ELF especificar).
    /// CRITICO: Implementacao real necessaria para evitar page fault/triple
    /// fault!
    pub fn map_kernel(
        &mut self,
        phys: u64,
        virt: u64,
        pages: usize,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        if phys % 4096 != 0 || virt % 4096 != 0 {
            return Err(BootError::Memory(MemoryError::InvalidAlignment));
        }

        // Mapear cada pagina individualmente
        for i in 0..pages {
            let page_phys = phys + (i as u64 * 4096);
            let page_virt = virt + (i as u64 * 4096);

            self.map_page(
                page_phys,
                page_virt,
                PAGE_PRESENT | PAGE_WRITABLE,
                allocator,
            )?;
        }
        Ok(())
    }

    /// Mapeia uma unica página 4KB: Virtual -> Fisica
    /// Cria tables intermediarias (PDP, PD, PT) sob demanda
    fn map_page(
        &mut self,
        phys: u64,
        virt: u64,
        flags: u64,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // Extrair indices da hierarquia de paginacao (4-level)
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        // Acessar PML4
        let pml4 = unsafe { &mut *(self.pml4_phys_addr as *mut [u64; 512]) };

        // Nivel 3: PDPT (Page Directory Pointer Table)
        let pdpt_addr = if pml4[pml4_idx] & PAGE_PRESENT != 0 {
            pml4[pml4_idx] & 0x000F_FFFF_FFFF_F000 // Mascara para endereco
        } else {
            let new_pdpt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pdpt as *mut u8, 0, 4096);
            }
            pml4[pml4_idx] = new_pdpt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pdpt
        };

        let pdpt = unsafe { &mut *(pdpt_addr as *mut [u64; 512]) };

        // Nivel 2: PD (Page Directory)
        let pd_addr = if pdpt[pdpt_idx] & PAGE_PRESENT != 0 {
            pdpt[pdpt_idx] & 0x000F_FFFF_FFFF_F000
        } else {
            let new_pd = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pd as *mut u8, 0, 4096);
            }
            pdpt[pdpt_idx] = new_pd | PAGE_PRESENT | PAGE_WRITABLE;
            new_pd
        };

        let pd = unsafe { &mut *(pd_addr as *mut [u64; 512]) };

        // Nivel 1: PT (Page Table)
        let pt_addr = if pd[pd_idx] & PAGE_PRESENT != 0 {
            pd[pd_idx] & 0x000F_FFFF_FFFF_F000
        } else {
            let new_pt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
            }
            pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pt
        };

        let pt = unsafe { &mut *(pt_addr as *mut [u64; 512]) };

        // Nivel 0: Mapear a pagina final
        pt[pt_idx] = phys | flags;

        Ok(())
    }
}
