//! Carregador de segmentos ELF na memória

use goblin::elf::Elf;

use crate::{
    core::{
        error::{BootError, ElfError, Result},
        types::LoadedKernel,
    },
    elf::ElfParser,
    memory::MemoryAllocator,
};

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
            "Carregando kernel: VAddr={:#x}-{:#x} Tam={:#x} ({} páginas)",
            min_vaddr,
            max_vaddr,
            kernel_size,
            kernel_pages
        );

        // IMPORTANTE: Alocar em QUALQUER endereço disponível, não em min_vaddr!
        // O kernel pode ter min_vaddr = 0x0, que não é válido para alocar.
        // Vamos alocar onde UEFI permitir e ajustar os offsets.
        let kernel_base_ptr = self.allocator.allocate_any(kernel_pages)?;

        log::info!(
            "Kernel alocado em {:#x} (VAddr original era {:#x})",
            kernel_base_ptr,
            min_vaddr
        );

        // Zerar memória (BSS e paddings)
        unsafe {
            self.allocator
                .zero_memory(kernel_base_ptr, kernel_size as usize);
        }

        // Copiar segmentos
        self.load_segments(&elf, elf_data, min_vaddr, kernel_base_ptr)?;

        log::info!("Kernel carregado com sucesso em {:#x}", kernel_base_ptr);

        // IMPORTANTE: Ajustar entry point para endereço físico!
        // O ELF tem entry point Virtual (ex: 0x1000).
        // Precisamos calcular o offset dentro da imagem e somar à base física.
        // Offset = VirtualAddr - MinVirtualAddr
        // EndereçoReal = BaseFísica + Offset

        let entry_offset = elf.entry - min_vaddr;
        let physical_entry = kernel_base_ptr + entry_offset;

        log::info!(
            "Entry point: ELF Virtual={:#x} - MinVAddr={:#x} -> Offset={:#x} -> Físico={:#x}",
            elf.entry,
            min_vaddr,
            entry_offset,
            physical_entry
        );

        Ok(LoadedKernel {
            base_address: kernel_base_ptr,
            size:         kernel_size,
            entry_point:  physical_entry, // Usar entry point físico!
        })
    }

    /// Carrega os segmentos PT_LOAD do ELF na memória
    fn load_segments(
        &self,
        elf: &Elf,
        elf_data: &[u8],
        min_vaddr: u64,
        physical_base: u64,
    ) -> Result<()> {
        for ph in &elf.program_headers {
            if ph.p_type == goblin::elf::program_header::PT_LOAD {
                let mem_size = ph.p_memsz as usize;
                let file_size = ph.p_filesz as usize;
                let vaddr = ph.p_vaddr;
                let offset = ph.p_offset as usize;

                // Calcular offset relativo ao base virtual do kernel
                let segment_offset = (vaddr - min_vaddr) as usize;

                // Endereço físico real = base física + offset
                let dest_ptr = (physical_base + segment_offset as u64) as *mut u8;

                log::info!(
                    "Segmento: VAddr={:#x} -> PAddr={:#x} FileSz={:#x} MemSz={:#x}",
                    vaddr,
                    dest_ptr as u64,
                    file_size,
                    mem_size
                );

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
