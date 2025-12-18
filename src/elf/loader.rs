//! Carregador de segmentos ELF na memória

use crate::elf::ElfParser;
use crate::error::{BootError, ElfError, Result};
use crate::memory::MemoryAllocator;
use crate::types::LoadedKernel;
use goblin::elf::Elf;


/// Carregador de ELF
pub struct ElfLoader<'a> {
    allocator: &'a MemoryAllocator<'a>,
}

impl<'a> ElfLoader<'a> {
    /// Cria um novo carregador de ELF
    pub fn new(allocator: &'a MemoryAllocator<'a>) -> Self {
        Self { allocator }
    }

    /// Carrega um arquivo ELF na memória
    ///
    /// # Argumentos
    /// * `elf_data` - Dados do arquivo ELF
    ///
    /// # Retorna
    /// Informações sobre o kernel carregado
    pub fn load(&self, elf_data: &[u8]) -> Result<LoadedKernel> {
        // Parsear ELF
        let elf = ElfParser::parse(elf_data)?;

        // Calcular intervalo de endereços
        let (min_vaddr, max_vaddr) = ElfParser::calculate_address_range(&elf);
        let kernel_size = max_vaddr - min_vaddr;
        let kernel_pages = MemoryAllocator::pages_for_size(kernel_size as usize);

        log::info!(
            "Alocando kernel contíguo: Base={:#x} Tam={:#x} ({} páginas)",
            min_vaddr,
            kernel_size,
            kernel_pages
        );

        // Alocar memória contígua para todo o kernel
        let kernel_base_ptr = self
            .allocator
            .allocate_at_address(min_vaddr, kernel_pages)?;

        // Zerar memória (BSS e paddings)
        unsafe {
            self.allocator.zero_memory(kernel_base_ptr, kernel_size as usize);
        }

        // Copiar segmentos
        self.load_segments(&elf, elf_data)?;

        log::info!("Kernel carregado com sucesso em {:#x}", kernel_base_ptr);

        Ok(LoadedKernel {
            base_address: kernel_base_ptr,
            size: kernel_size,
            entry_point: elf.entry,
        })
    }

    /// Carrega os segmentos PT_LOAD do ELF na memória
    fn load_segments(&self, elf: &Elf, elf_data: &[u8]) -> Result<()> {
        for ph in &elf.program_headers {
            if ph.p_type == goblin::elf::program_header::PT_LOAD {
                let mem_size = ph.p_memsz as usize;
                let file_size = ph.p_filesz as usize;
                let vaddr = ph.p_vaddr;
                let offset = ph.p_offset as usize;

                log::info!(
                    "Copiando segmento: VAddr={:#x} FileSz={:#x} MemSz={:#x}",
                    vaddr,
                    file_size,
                    mem_size
                );

                let dest_ptr = vaddr as *mut u8;

                // Copiar conteúdo do arquivo
                if file_size > 0 {
                    if offset + file_size > elf_data.len() {
                        return Err(BootError::Elf(ElfError::SegmentCopyError));
                    }

                    let src_slice = &elf_data[offset..offset + file_size];
                    unsafe {
                        let dest_slice = core::slice::from_raw_parts_mut(dest_ptr, file_size);
                        dest_slice.copy_from_slice(src_slice);
                    }
                }

                // Nota: Resto da memória (mem_size - file_size) já foi zerado
            }
        }

        Ok(())
    }
}
