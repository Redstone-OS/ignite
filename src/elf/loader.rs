//! Carregador de Segmentos ELF com Suporte a Paginação
//!
//! Responsável por ler os segmentos `PT_LOAD`, alocar frames físicos
//! correspondentes e mapeá-los no endereço virtual solicitado pelo Kernel.

use goblin::elf::{Elf, program_header::PT_LOAD};

use super::header::validate_header;
use crate::{
    core::{
        error::{BootError, ElfError, Result},
        types::LoadedKernel,
    },
    memory::{FrameAllocator, PageTableManager, layout::PAGE_SIZE},
};

/// Carregador de Kernel ELF.
pub struct ElfLoader<'a, A: FrameAllocator> {
    allocator:  &'a mut A,
    page_table: &'a mut PageTableManager,
}

impl<'a, A: FrameAllocator> ElfLoader<'a, A> {
    /// Cria um novo carregador vinculado a um alocador e uma tabela de páginas.
    pub fn new(allocator: &'a mut A, page_table: &'a mut PageTableManager) -> Self {
        Self {
            allocator,
            page_table,
        }
    }

    /// Carrega, aloca e mapeia o Kernel na memória.
    ///
    /// # Processo Industrial
    /// 1. Parse e Validação do Header.
    /// 2. Iteração de Segmentos `PT_LOAD`.
    /// 3. Alocação de Frames Físicos (sob demanda).
    /// 4. Cópia de Dados (Arquivo -> RAM Física).
    /// 5. Zeroização de BSS (RAM Física).
    /// 6. Mapeamento (Page Tables: Virtual -> Física).
    pub fn load_kernel(&mut self, file_data: &[u8]) -> Result<LoadedKernel> {
        let elf = Elf::parse(file_data).map_err(|_| BootError::Elf(ElfError::ParseError))?;
        validate_header(&elf.header)?;

        log::info!("Iniciando carregamento do Kernel ELF...");

        let mut kernel_phys_start = u64::MAX;
        let mut kernel_phys_end = 0;
        let mut kernel_virt_start = u64::MAX;
        let mut kernel_virt_end = 0;

        for ph in elf.program_headers.iter() {
            if ph.p_type != PT_LOAD {
                continue;
            }

            if ph.p_memsz == 0 {
                continue;
            }

            // Endereços Virtuais do Segmento
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

            // Alinhamento de Páginas (4KiB)
            // O segmento pode começar no meio de uma página (ex: 0x10080),
            // mas alocamos e mapeamos a página inteira (0x10000).
            let page_offset = virt_start % PAGE_SIZE;
            let virt_page_start = virt_start - page_offset;
            let total_bytes_needed = (virt_end - virt_page_start) as usize;
            let pages_needed = (total_bytes_needed + (PAGE_SIZE as usize - 1)) / PAGE_SIZE as usize;

            log::debug!(
                "Carregando Segmento: Virt={:#x} Tam={:#x} ({} Páginas)",
                virt_start,
                ph.p_memsz,
                pages_needed
            );

            // 1. Alocar memória física
            let phys_addr = self.allocator.allocate_frame(pages_needed)?;

            // Rastrear limites físicos e virtuais globais do kernel
            if phys_addr < kernel_phys_start {
                kernel_phys_start = phys_addr;
            }
            let phys_end = phys_addr + (pages_needed as u64 * PAGE_SIZE);
            if phys_end > kernel_phys_end {
                kernel_phys_end = phys_end;
            }

            if virt_start < kernel_virt_start {
                kernel_virt_start = virt_start;
            }
            if virt_end > kernel_virt_end {
                kernel_virt_end = virt_end;
            }

            // 2. Mapear na Tabela de Páginas (Virtual -> Físico)
            // Isso cria a "ponte" para o kernel rodar no endereço que ele espera.
            self.page_table
                .map_kernel(phys_addr, virt_page_start, pages_needed, self.allocator)?;

            // 3. Copiar Dados (Arquivo -> RAM)
            // Nota: Escrevemos diretamente na memória física.
            unsafe {
                let dest_ptr = (phys_addr + page_offset) as *mut u8;

                // Copiar parte presente no arquivo
                if file_size > 0 {
                    core::ptr::copy_nonoverlapping(
                        file_data.as_ptr().add(file_start),
                        dest_ptr,
                        file_size,
                    );
                }

                // 4. Zerar BSS (Memória restante do segmento não presente no arquivo)
                if ph.p_memsz > ph.p_filesz {
                    let bss_start_ptr = dest_ptr.add(file_size);
                    let bss_size = (ph.p_memsz - ph.p_filesz) as usize;
                    core::ptr::write_bytes(bss_start_ptr, 0, bss_size);
                }
            }
        }

        // Calcular Entry Point
        // O Entry Point no header ELF é virtual. Como já mapeamos tudo,
        // o kernel pode pular direto para esse endereço virtual DEPOIS de carregar o
        // CR3.
        let entry_point = elf.entry;

        log::info!("Kernel Carregado. Entry Point Virtual: {:#x}", entry_point);
        log::info!(
            "Memória Física Ocupada: {:#x} - {:#x}",
            kernel_phys_start,
            kernel_phys_end
        );

        Ok(LoadedKernel {
            base_address: kernel_phys_start, // Base física (para info)
            size: kernel_phys_end - kernel_phys_start,
            entry_point, // Entry point virtual (para salto)
        })
    }
}
