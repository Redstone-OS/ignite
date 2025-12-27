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

/// Flags x86_64 para page table entries
/// ====================================
/// Estas flags são usadas tanto para páginas 4KiB quanto para huge pages 2MiB.
/// Durante split de huge page, todas as flags relevantes devem ser preservadas.

// Flags básicas (presentes em todos os níveis)
const PAGE_PRESENT: u64 = 1 << 0; // P - Página presente
const PAGE_WRITABLE: u64 = 1 << 1; // R/W - Leitura/Escrita
const PAGE_USER: u64 = 1 << 2; // U/S - User/Supervisor
const PAGE_PWT: u64 = 1 << 3; // Page-level Write-Through
const PAGE_PCD: u64 = 1 << 4; // Page-level Cache Disable
const PAGE_ACCESSED: u64 = 1 << 5; // A - Accessed
const PAGE_DIRTY: u64 = 1 << 6; // D - Dirty (para PT/huge pages)
const PAGE_HUGE: u64 = 1 << 7; // PS - Page Size (2MiB em PD, 1GiB em PDPT)
const PAGE_GLOBAL: u64 = 1 << 8; // G - Global (não flush no CR3 reload)
#[allow(dead_code)]
const PAGE_PAT: u64 = 1 << 12; // PAT (para huge pages; bit 7 em PT)
const PAGE_NO_EXEC: u64 = 1 << 63; // NX - No Execute

/// Máscara para flags que devem ser preservadas ao converter huge page →
/// páginas 4KiB. Inclui: Present, Writable, User, PWT, PCD, Accessed, Dirty,
/// Global, NX NÃO inclui: PAGE_HUGE (será removida), PAGE_PAT (posição
/// diferente em 4KiB)
const PRESERVED_FLAGS_MASK: u64 = PAGE_PRESENT
    | PAGE_WRITABLE
    | PAGE_USER
    | PAGE_PWT
    | PAGE_PCD
    | PAGE_ACCESSED
    | PAGE_DIRTY
    | PAGE_GLOBAL
    | PAGE_NO_EXEC;

/// Máscara para extrair PAT de huge page (bit 12)
const HUGE_PAGE_PAT_BIT: u64 = 1 << 12;

/// Máscara para PAT em página 4KiB (bit 7, mas PAGE_HUGE não existe aqui)
const PAGE_PAT_4K: u64 = 1 << 7;

// Máscara para extrair endereço base de uma entrada de page table
const ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

/// Tamanho de uma huge page (2MiB)
#[allow(dead_code)]
const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;

/// Tamanho de uma página normal (4KiB)
const PAGE_SIZE: u64 = 4096;

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

    /// Garante que o identity map para `phys_addr` use páginas 4KiB (não huge
    /// page).
    ///
    /// Se a huge page correspondente no identity map ainda existir, ela será
    /// dividida em 512 páginas de 4KiB preservando o mapeamento.
    ///
    /// **Use case:** O kernel precisa acessar certos endereços físicos via
    /// identity map para estruturas como page tables, BootInfo, etc. Huge
    /// pages impedem mapeamento granular, então esta função garante acesso
    /// correto.
    pub fn ensure_identity_map_4k(
        &mut self,
        phys_addr: u64,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        // No identity map, virt == phys
        self.map_page(
            phys_addr,
            phys_addr,
            PAGE_PRESENT | PAGE_WRITABLE,
            allocator,
        )
    }

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

    /// Mapeia memória física de 0 até `max_phys_addr` usando huge pages (2
    /// MiB).
    ///
    /// **Motivação:** o bootloader e o early-kernel frequentemente executam em
    /// endereços físicos que podem estar em qualquer região da RAM. Ao trocar
    /// CR3 precisamos garantir que esses endereços permaneçam acessíveis.
    /// Usar huge pages reduz TLB pressure e diminui a quantidade de page
    /// tables alocadas.
    ///
    /// **Uso:** Para sistemas com mais de 4GB de RAM, passe o endereço físico
    /// máximo + margem para garantir que toda a memória esteja mapeada.
    pub fn identity_map_range(
        &mut self,
        max_phys_addr: u64,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        const SIZE_2MIB: u64 = 0x20_0000;

        // Arredondar para cima para o próximo boundary de 2MiB
        let aligned_max = (max_phys_addr + SIZE_2MIB - 1) & !(SIZE_2MIB - 1);

        // Usar huge pages (2MiB) para performance.
        let mut phys = 0u64;
        while phys < aligned_max {
            self.map_huge_page(phys, phys, PAGE_PRESENT | PAGE_WRITABLE, allocator)?;
            phys = phys.wrapping_add(SIZE_2MIB);
        }

        Ok(())
    }

    /// Mapeia os primeiros 4 GiB de memória física usando huge pages (2 MiB).
    ///
    /// **Nota:** Para sistemas com mais de 4GB de RAM, use `identity_map_range`
    /// com o endereço físico máximo do memory map para evitar page faults.
    ///
    /// Mantido para compatibilidade com código existente.
    pub fn identity_map_4gib(
        &mut self,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<()> {
        const SIZE_4GIB: u64 = 0x1_0000_0000;
        self.identity_map_range(SIZE_4GIB, allocator)
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
    // Split de Huge Page (Atômico e Completo)
    // ---------------------------------------------------------------------
    //
    // Quando o bootloader precisa mapear páginas 4KiB em uma região que já
    // possui uma huge page de 2MiB, é necessário "dividir" essa huge page em
    // 512 páginas normais de 4KiB.
    //
    // **CRÍTICO**: Todas as 512 páginas DEVEM ser preenchidas para manter
    // o identity map válido. Não preencher páginas deixa "buracos" no
    // mapeamento que causam Page Faults quando o kernel tenta acessar
    // essas regiões.
    //
    // **Flags preservadas**:
    // - Present, Writable, User, PWT, PCD, Accessed, Dirty, Global, NX
    // - PAT é convertido da posição de huge page (bit 12) para 4KiB (bit 7)

    /// Divide uma huge page de 2MiB em 512 páginas de 4KiB.
    ///
    /// Esta função:
    /// 1. Extrai o endereço base e flags da huge page existente
    /// 2. Aloca um frame para a nova Page Table
    /// 3. Preenche TODAS as 512 entradas preservando flags
    /// 4. Substitui a entrada de huge page pela nova PT
    ///
    /// # Rollback
    /// Se a alocação falhar, a huge page original permanece inalterada.
    ///
    /// # Returns
    /// O endereço físico da nova Page Table alocada.
    fn split_huge_page_to_pt(
        pd: &mut [u64; 512],
        pd_idx: usize,
        allocator: &mut (impl FrameAllocator + ?Sized),
    ) -> Result<u64> {
        let huge_entry = pd[pd_idx];

        // Verificar se realmente é uma huge page
        if huge_entry & PAGE_HUGE == 0 {
            // Não é huge page, retorna PT existente
            return Ok(huge_entry & ADDR_MASK);
        }

        // Log: huge page será dividida
        let _huge_phys = huge_entry & ADDR_MASK;

        // Extrair endereço base da huge page (alinhado a 2MiB)
        let huge_phys_base = huge_entry & ADDR_MASK;

        // Extrair flags que devem ser preservadas
        let mut preserved_flags = huge_entry & PRESERVED_FLAGS_MASK;

        // Converter PAT: em huge page está no bit 12, em 4KiB vai para bit 7
        if huge_entry & HUGE_PAGE_PAT_BIT != 0 {
            preserved_flags |= PAGE_PAT_4K;
        }

        // Alocar frame para a nova Page Table
        // Se falhar, a huge page original permanece inalterada (rollback implícito)
        let new_pt_phys = allocator.allocate_frame(1)?;

        // Preencher TODAS as 512 entradas da PT
        unsafe {
            let pt = new_pt_phys as *mut [u64; 512];

            for i in 0..512 {
                // Calcular endereço físico desta página de 4KiB
                let page_phys = huge_phys_base + (i as u64 * PAGE_SIZE);

                // Criar entrada com endereço + flags preservadas
                // NÃO incluímos PAGE_HUGE (é uma página 4KiB agora)
                (*pt)[i] = (page_phys & ADDR_MASK) | preserved_flags;
            }
        }

        // Substituir entrada de huge page pela nova PT
        // Flags da entrada de PD: Present + Writable (+ User se aplicável)
        let pd_flags = (huge_entry & (PAGE_PRESENT | PAGE_WRITABLE | PAGE_USER))
            | PAGE_PRESENT
            | PAGE_WRITABLE;
        pd[pd_idx] = new_pt_phys | pd_flags;

        Ok(new_pt_phys)
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
            // Se for huge page, precisamos fazer split para páginas 4KiB
            if pd[pd_idx] & PAGE_HUGE != 0 {
                // Split atômico de huge page → 512 páginas de 4KiB
                Self::split_huge_page_to_pt(pd, pd_idx, allocator)?
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
        let pt_phys = if pd[pd_idx] & PAGE_PRESENT != 0 {
            if pd[pd_idx] & PAGE_HUGE != 0 {
                // Huge page precisa ser dividida - usar função de split completo
                Self::split_huge_page_to_pt(pd, pd_idx, allocator)?;
            }
            pd[pd_idx] & ADDR_MASK
        } else {
            let new_pt = allocator.allocate_frame(1)?;
            unsafe {
                core::ptr::write_bytes(new_pt as *mut u8, 0, 4096);
            }
            pd[pd_idx] = new_pt | PAGE_PRESENT | PAGE_WRITABLE;
            new_pt
        };

        // CRÍTICO: Garantir que a PT do scratch esteja acessível via identity map.
        // O kernel usa phys_to_virt(SCRATCH_PT_PHYS) para mapear frames no scratch,
        // então a PT deve estar em uma região do identity map com páginas 4KiB.
        self.ensure_identity_map_4k(pt_phys, allocator)?;

        Ok(())
    }
}
