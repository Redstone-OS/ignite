//! Gerenciamento de Tabelas de Página (x86_64 — Paging)
//! --------------------------------------------------
//!
//! Este módulo provê um `PageTableManager` minimalista e prático para criar e
//! manipular uma hierarquia de paginação em 4 níveis (PML4 → PDPT → PD → PT).
//! É pensado para uso no bootloader / early-boot do Redstone OS: cria um PML4
//! raiz limpo, mapeia uma região *identity* (0..4GiB) com huge pages de 2MiB,
//! e prepara um *scratch slot* (endereço virtual fixo) que o kernel usa para
//! mapear frames temporários sem conflitar com huge pages.
//!
//! ### Objetivos
//! - Garantir que, após trocar CR3 para a nova PML4, o código do bootloader
//!   continue acessível (identity map dos primeiros 4 GiB).
//! - Reduzir TLB pressure onde faz sentido usando huge pages (2MiB).
//! - Fornecer um scratch slot seguro para operações do kernel que precisam
//!   mapear frames 4KiB mesmo quando huge pages cobrem a mesma região física.
//!
//! ### Contratos e Invariantes IMPORTANTES
//! 1. **Identity map 4GiB deve ser criado antes** de qualquer operação crítica
//!    que dependa de acessar memória física baixa depois da troca de CR3.
//! 2. **setup_scratch_slot** deve ser chamado **após** o identity_map_4gib e
//!    **antes** do kernel começar a criar suas próprias page tables.
//! 3. Todos os endereços físicos/virtuais mapeados aqui devem estar alinhados a
//!    4KiB (ou 2MiB para huge pages). Funções validam alinhamento quando
//!    aplicável.
//!
//! ### Segurança / `unsafe`
//! Este código manipula endereços físicos interpretados como ponteiros. Essas
//! conversões são intrinsecamente `unsafe`. Os `unsafe` blocks aqui assumem:
//! - O frame físico retornado pelo `FrameAllocator` tem pelo menos 4096 bytes.
//! - As operações de escrita (zeroing) são seguras para o range alocado.
//! - Os índices calculados (PML4/PDPT/PD/PT) não ultrapassam 0..512.
//!
//! Se for necessário robustecer, substitua `core::ptr::write_bytes` por
//! wrappers que validem tamanhos e erros, e considere abstrair memória física
//! para uma camada que trate bounds-checks/ASLR/relocations quando apropriado.
//!
//! ### TODO / Melhoria futura
//! - Transformar duplicação de código (obter/criar table entries) em helper
//!   `get_or_create_table(level, idx)` para reduzir repetição.
//! - Suportar flags adicionais (NX, PAT, user/supervisor, cache attribs).
//! - Registrar métricas (quantos frames alocados para page tables) para
//!   debugging.
//! - Tratar erros com variantes mais específicas de `BootError` em vez de
//!   `expect`.

use super::allocator::FrameAllocator;
use crate::core::error::{BootError, MemoryError, Result};

/// Flags x86_64 (básicas usadas neste módulo)
const PAGE_PRESENT: u64 = 1 << 0;
const PAGE_WRITABLE: u64 = 1 << 1;
const PAGE_HUGE: u64 = 1 << 7; // PD entry com este bit = 2MiB page (ou 1GiB em PDPT)
const PAGE_NO_EXEC: u64 = 1 << 63; // NX bit (quando suportado)

// Máscara para extrair endereço base de uma entrada de page table (bits de
// endereço)
const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

/// Gerenciador de Tabelas de Página.
///
/// Mantém apenas o endereço físico da PML4 raiz e métodos para criar
/// mapeamentos (identity, huge pages e páginas 4KiB individuais). Não realiza a
/// carga de CR3 — essa responsabilidade fica com quem instancia/usa
/// `PageTableManager`.
pub struct PageTableManager {
    pml4_phys_addr: u64,
}

impl PageTableManager {
    /// Cria uma nova PML4 limpa (um frame alocado) e retorna o gerenciador.
    ///
    /// - `allocator` é usado para alocar o frame que conterá a PML4.
    /// - A PML4 é zerada por segurança, evitando herdar lixo da RAM.
    pub fn new(allocator: &mut impl FrameAllocator) -> Result<Self> {
        // 1) Aloca frame para a PML4 raiz
        let pml4 = allocator.allocate_frame(1)?;

        // 2) Zera a página (segurança: não herdar dados)
        unsafe {
            let ptr = pml4 as *mut u64;
            // 512 entradas * 8 bytes = 4096 bytes
            core::ptr::write_bytes(ptr, 0, 512);
        }

        Ok(Self {
            pml4_phys_addr: pml4,
        })
    }

    /// Retorna o endereço físico da PML4 (útil para carregar em CR3).
    pub fn pml4_addr(&self) -> u64 {
        self.pml4_phys_addr
    }

    // ---------------------------------------------------------------------
    // Identity map (general-purpose)
    // ---------------------------------------------------------------------

    /// Mapeia `count` páginas (4KiB) começando em `phys_addr` onde virtual ==
    /// físico.
    ///
    /// Use para criar mapeamentos finos (4KiB). Para grandes faixas (ex.:
    /// 0..4GiB) prefira `identity_map_4gib` que usa huge pages de 2MiB por
    /// performance.
    pub fn identity_map(
        &mut self,
        phys_addr: u64,
        count: usize,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        for i in 0..count {
            let addr = phys_addr + (i as u64 * 4096);
            self.map_page(addr, addr, PAGE_PRESENT | PAGE_WRITABLE, allocator)?;
        }
        Ok(())
    }

    /// Mapeia os primeiros 4 GiB de memória física usando huge pages (2 MiB).
    ///
    /// **Motivação:** o bootloader e o early-kernel frequentemente executam em
    /// endereços físicos baixos. Ao trocar CR3 precisamos garantir que esses
    /// endereços permaneçam acessíveis. Usar huge pages reduz TLB pressure
    /// e diminui a quantidade de page tables alocadas.
    pub fn identity_map_4gib(
        &mut self,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        const SIZE_4GIB: u64 = 0x1_0000_0000;
        const SIZE_2MIB: u64 = 0x20_0000;

        let mut phys = 0u64;
        while phys < SIZE_4GIB {
            self.map_huge_page(phys, phys, PAGE_PRESENT | PAGE_WRITABLE, allocator)?;
            phys = phys.wrapping_add(SIZE_2MIB);
        }

        Ok(())
    }

    // ---------------------------------------------------------------------
    // Mapeamentos de Huge Page (2MiB)
    // ---------------------------------------------------------------------

    /// Mapeia uma huge page (2MiB) de `phys` para `virt` com `flags`.
    ///
    /// Este método garante que PML4 → PDPT → PD existam (criando frames quando
    /// necessário) e então escreve a entrada de PD com o bit PAGE_HUGE.
    fn map_huge_page(
        &mut self,
        phys: u64,
        virt: u64,
        flags: u64,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // Índices na hierarquia x86_64
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;

        let pml4 = unsafe { &mut *(self.pml4_phys_addr as *mut [u64; 512]) };

        // PDPT
        let pdpt_addr = if pml4[pml4_idx] & PAGE_PRESENT != 0 {
            pml4[pml4_idx] & ADDR_MASK
        } else {
            let new_pdpt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pdpt as *mut u8, 0, 4096);
            }
            pml4[pml4_idx] = new_pdpt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pdpt
        };
        let pdpt = unsafe { &mut *(pdpt_addr as *mut [u64; 512]) };

        // PD
        let pd_addr = if pdpt[pdpt_idx] & PAGE_PRESENT != 0 {
            pdpt[pdpt_idx] & ADDR_MASK
        } else {
            let new_pd = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pd as *mut u8, 0, 4096);
            }
            pdpt[pdpt_idx] = new_pd | PAGE_PRESENT | PAGE_WRITABLE;
            new_pd
        };
        let pd = unsafe { &mut *(pd_addr as *mut [u64; 512]) };

        // Escrever entry de PD como huge page (2MiB)
        pd[pd_idx] = (phys & ADDR_MASK) | flags | PAGE_HUGE;

        Ok(())
    }

    // ---------------------------------------------------------------------
    // Mapear Kernel / páginas 4KiB
    // ---------------------------------------------------------------------

    /// Mapeia o kernel (ou qualquer região) em páginas 4KiB.
    ///
    /// - `phys` e `virt` devem estar alinhados a 4 KiB.
    /// - `pages` é o número de páginas de 4 KiB a mapear.
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

    /// Mapeia uma página 4KiB: cria tables intermediárias sob demanda.
    ///
    /// Este é o método "workhorse" para mapeamentos finos. Ele:
    /// - calcula os índices PML4/PDPT/PD/PT,
    /// - garante que cada nível exista (alocando frames e zerando),
    /// - escreve a entrada final na PT com `phys | flags`.
    fn map_page(
        &mut self,
        phys: u64,
        virt: u64,
        flags: u64,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // Índices (4-level paging)
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx = ((virt >> 12) & 0x1FF) as usize;

        // PML4
        let pml4 = unsafe { &mut *(self.pml4_phys_addr as *mut [u64; 512]) };

        // PDPT
        let pdpt_addr = if pml4[pml4_idx] & PAGE_PRESENT != 0 {
            pml4[pml4_idx] & ADDR_MASK
        } else {
            let new_pdpt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pdpt as *mut u8, 0, 4096);
            }
            pml4[pml4_idx] = new_pdpt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pdpt
        };
        let pdpt = unsafe { &mut *(pdpt_addr as *mut [u64; 512]) };

        // PD
        let pd_addr = if pdpt[pdpt_idx] & PAGE_PRESENT != 0 {
            pdpt[pdpt_idx] & ADDR_MASK
        } else {
            let new_pd = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pd as *mut u8, 0, 4096);
            }
            pdpt[pdpt_idx] = new_pd | PAGE_PRESENT | PAGE_WRITABLE;
            new_pd
        };
        let pd = unsafe { &mut *(pd_addr as *mut [u64; 512]) };

        // PT (não queremos uma huge page aqui — garantimos PT normal)
        let pt_addr = if pd[pd_idx] & PAGE_PRESENT != 0 {
            // Se for huge page, isso significa conflito — substituímos pela PT
            if pd[pd_idx] & PAGE_HUGE != 0 {
                // Conflito: havia uma huge page. Substituímos por uma PT normal.
                // Opcional: poderíamos herdá-la ou reproteger; aqui simplesmente criamos PT.
                let new_pt = allocator.allocate_frame(1)?;
                unsafe {
                    core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
                }
                pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
                new_pt
            } else {
                pd[pd_idx] & ADDR_MASK
            }
        } else {
            let new_pt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
            }
            pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pt
        };

        let pt = unsafe { &mut *(pt_addr as *mut [u64; 512]) };

        // Entrada final: mapear a página
        pt[pt_idx] = (phys & ADDR_MASK) | flags;

        Ok(())
    }

    // ---------------------------------------------------------------------
    // Scratch slot — área virtual fixa para uso do kernel
    // ---------------------------------------------------------------------
    //
    // Explicação rápida:
    // - Identity map usa huge pages (2MiB) cobrindo região baixa.
    // - Kernel precisa, ocasionalmente, mapear frames 4KiB para
    //   zerá-los/initializar.
    // - Em regiões com huge pages, a PT de 4KiB não é consultada; isso causa GPF.
    // - O scratch slot fornece um endereço virtual limpo (sem huge pages) onde o
    //   kernel pode temporariamente mapear frames 4KiB.
    //
    // **Contrato:** chamada DEVE ocorrer após `identity_map_4gib()` e antes de
    // qualquer operação do kernel que espere poder mapear frames 4KiB.
    //
    /// Configura o scratch slot (região virtual fixa) que o kernel usa para
    /// mapear frames temporários.
    pub fn setup_scratch_slot(
        &mut self,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // Endereço virtual acordado entre bootloader e kernel.
        // PML4[508] (ou 509 em designs alternativos) — manter sincronizado com kernel.
        const SCRATCH_VIRT: u64 = 0xFFFF_FE00_0000_0000;

        let pml4_idx = ((SCRATCH_VIRT >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((SCRATCH_VIRT >> 30) & 0x1FF) as usize;
        let pd_idx = ((SCRATCH_VIRT >> 21) & 0x1FF) as usize;

        let pml4 = unsafe { &mut *(self.pml4_phys_addr as *mut [u64; 512]) };

        // PDPT
        let pdpt_addr = if pml4[pml4_idx] & PAGE_PRESENT != 0 {
            pml4[pml4_idx] & ADDR_MASK
        } else {
            let new_pdpt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pdpt as *mut u8, 0, 4096);
            }
            pml4[pml4_idx] = new_pdpt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pdpt
        };
        let pdpt = unsafe { &mut *(pdpt_addr as *mut [u64; 512]) };

        // PD
        let pd_addr = if pdpt[pdpt_idx] & PAGE_PRESENT != 0 {
            pdpt[pdpt_idx] & ADDR_MASK
        } else {
            let new_pd = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pd as *mut u8, 0, 4096);
            }
            pdpt[pdpt_idx] = new_pd | PAGE_PRESENT | PAGE_WRITABLE;
            new_pd
        };
        let pd = unsafe { &mut *(pd_addr as *mut [u64; 512]) };

        // PT: garantir que existe uma PT (não uma huge page).
        if pd[pd_idx] & PAGE_PRESENT != 0 {
            if pd[pd_idx] & PAGE_HUGE != 0 {
                // Havia uma huge page — substituímos por uma PT normal.
                let new_pt = allocator.allocate_frame(1)?;
                unsafe {
                    core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
                }
                pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
            }
            // Se já existe PT normal, nada a fazer.
        } else {
            let new_pt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
            }
            pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
        }

        Ok(())
    }
}
