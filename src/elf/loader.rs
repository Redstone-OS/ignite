//! Carregador de Segmentos ELF com Suporte a Paginação
//!
//! Lê segmentos `PT_LOAD`, aloca frames físicos correspondentes e mapeia
//! no endereço virtual solicitado pelo Kernel.

use goblin::elf::{Elf, program_header::PT_LOAD};

use super::header::validate_header;
use crate::{
    core::{
        error::{BootError, ElfError, Result},
        types::LoadedKernel,
    },
    memory::{FrameAllocator, PageTableManager, layout::PAGE_SIZE},
};

// ?Sized permite aceitar Trait Objects
pub struct ElfLoader<'a, A: FrameAllocator + ?Sized> {
    allocator:  &'a mut A,
    page_table: &'a mut PageTableManager,
}

impl<'a, A: FrameAllocator + ?Sized> ElfLoader<'a, A> {
    pub fn new(allocator: &'a mut A, page_table: &'a mut PageTableManager) -> Self {
        Self {
            allocator,
            page_table,
        }
    }

    /// Carrega, aloca e mapeia o Kernel na memória.
    ///
    /// # Passos
    /// 1. Parse e validação do header ELF.
    /// 2. Iteração de segmentos `PT_LOAD`.
    /// 3. Alocação de frames físicos (sob demanda).
    /// 4. Cópia de dados (arquivo -> RAM física).
    /// 5. Zeroização de BSS (memória restante do segmento).
    /// 6. Mapeamento (tabela de páginas: virtual -> física).
    pub fn load_kernel(&mut self, file_data: &[u8]) -> Result<LoadedKernel> {
        let elf = Elf::parse(file_data).map_err(|_| BootError::Elf(ElfError::ParseError))?;
        validate_header(&elf.header)?;

        let mut kernel_phys_start = u64::MAX;
        let mut kernel_phys_end = 0;
        let mut kernel_virt_start = u64::MAX;
        let mut kernel_virt_end = 0;

        for ph in elf.program_headers.iter() {
            if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
                continue;
            }

            // Endereços virtuais do segmento
            let virt_start = ph.p_vaddr;
            let virt_end = virt_start + ph.p_memsz;

            // Dados no arquivo
            let file_start = ph.p_offset as usize;
            let file_size = ph.p_filesz as usize;
            let file_end = file_start + file_size;

            // Validação de limites do arquivo
            if file_end > file_data.len() {
                return Err(BootError::Elf(ElfError::SegmentCopyError));
            }

            // Alinhamento de páginas
            let page_offset = virt_start % PAGE_SIZE;
            let virt_page_start = virt_start - page_offset;
            let total_bytes_needed = (virt_end - virt_page_start) as usize;
            let pages_needed = (total_bytes_needed + (PAGE_SIZE as usize - 1)) / PAGE_SIZE as usize;

            // Log de debug removido para output limpo

            // 1. Alocar memória física
            let phys_addr = self.allocator.allocate_frame(pages_needed)?;

            // Rastrear limites físicos
            if phys_addr < kernel_phys_start {
                kernel_phys_start = phys_addr;
            }
            let phys_end = phys_addr + (pages_needed as u64 * PAGE_SIZE);
            if phys_end > kernel_phys_end {
                kernel_phys_end = phys_end;
            }

            // Rastrear limites virtuais
            if virt_start < kernel_virt_start {
                kernel_virt_start = virt_start;
            }
            if virt_end > kernel_virt_end {
                kernel_virt_end = virt_end;
            }

            // 2. Mapear na tabela de páginas (virtual -> física)
            self.page_table
                .map_kernel(phys_addr, virt_page_start, pages_needed, self.allocator)?;

            // 3. CRÍTICO: Garantir que o identity map tenha páginas 4KiB para esta região
            // Isso permite que o kernel acesse memória física via phys_to_virt()
            for j in 0..pages_needed {
                let page_phys = phys_addr + (j as u64 * PAGE_SIZE);
                self.page_table
                    .ensure_identity_map_4k(page_phys, self.allocator)?;
            }

            // 3. Copiar dados e zeroizar BSS
            unsafe {
                let dest_ptr = (phys_addr + page_offset) as *mut u8;

                if file_size > 0 {
                    core::ptr::copy_nonoverlapping(
                        file_data.as_ptr().add(file_start),
                        dest_ptr,
                        file_size,
                    );
                }

                if ph.p_memsz > ph.p_filesz {
                    let bss_start_ptr = dest_ptr.add(file_size);
                    let bss_size = (ph.p_memsz - ph.p_filesz) as usize;
                    core::ptr::write_bytes(bss_start_ptr, 0, bss_size);
                }
            }
        }

        let entry_point = elf.entry;

        crate::println!(
            "[92m[1m[OK][0m Kernel carregado. Entry point virtual: {:#x}",
            entry_point
        );
        crate::println!(
            "[92m[1m[OK][0m Memória física ocupada: {:#x} - {:#x}",
            kernel_phys_start,
            kernel_phys_end
        );

        Ok(LoadedKernel {
            base_address: kernel_phys_start,
            // Mapear páginas extras para PMM bitmap
            size: {
                const EXTRA_PAGES: usize = 16;
                for i in 0..EXTRA_PAGES {
                    let extra_phys = kernel_phys_end + (i as u64 * PAGE_SIZE);
                    let _ = self
                        .page_table
                        .ensure_identity_map_4k(extra_phys, self.allocator);
                }
                kernel_phys_end - kernel_phys_start
            },
            entry_point,
        })
    }
}
