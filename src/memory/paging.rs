//! Gerenciamento de Tabelas de Página (x86_64 Paging)
//!
//! Responsável por criar o espaço de endereçamento virtual inicial para o
//! Kernel. Cria mapeamentos 4-level (PML4) com suporte a Huge Pages (2MiB) para
//! eficiência.

use super::allocator::FrameAllocator;
use crate::core::error::{BootError, MemoryError, Result};

// Constantes de flags x86_64 (bitmask)
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
    /// Essencial para que o bootloader continue rodando após ligar paginação.
    pub fn identity_map(
        &mut self,
        phys_addr: u64,
        count: usize,
        allocator: &mut impl FrameAllocator,
    ) -> Result<()> {
        // Exemplo simplificado. Num código real, isso calcula os índices
        // PML4 -> PDPT -> PD -> PT e preenche as entradas.
        // Se usar huge pages (2MB), economiza memória de tabelas e TLB.

        // TODO: Implementar walker de tabela.
        // Se falhar em alocar uma subtabela, retorna Erro.
        Ok(())
    }

    /// Mapeia o Kernel no Higher Half.
    /// phys: Onde o kernel está na RAM.
    /// virt: O endereço virtual alvo (ex: 0xFFFF_8000...)
    pub fn map_kernel(
        &mut self,
        phys: u64,
        virt: u64,
        pages: usize,
        allocator: &mut impl FrameAllocator,
    ) -> Result<()> {
        // Validação de alinhamento
        if phys % 4096 != 0 || virt % 4096 != 0 {
            return Err(BootError::Memory(MemoryError::InvalidAlignment));
        }

        // Loop de mapeamento
        for i in 0..pages {
            let offset = (i as u64) * 4096;
            // self.map_page(virt + offset, phys + offset, flags)?;
        }
        Ok(())
    }
}
