//! Parser de arquivos ELF

use crate::error::{BootError, ElfError, Result};
use goblin::elf::Elf;

/// Parser de arquivos ELF
pub struct ElfParser;

impl ElfParser {
    /// Parseia um arquivo ELF
    ///
    /// # Argumentos
    /// * `data` - Dados do arquivo ELF
    ///
    /// # Retorna
    /// Estrutura Elf parseada
    pub fn parse(data: &[u8]) -> Result<Elf<'_>> {
        log::info!("Parseando arquivo ELF...");

        let elf = Elf::parse(data).map_err(|_| BootError::Elf(ElfError::ParseError))?;

        // Validar ponto de entrada
        if elf.entry == 0 {
            return Err(BootError::Elf(ElfError::InvalidEntryPoint));
        }

        // Verificar se há segmentos carregáveis
        let has_loadable = elf
            .program_headers
            .iter()
            .any(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD);

        if !has_loadable {
            return Err(BootError::Elf(ElfError::NoLoadableSegments));
        }

        log::info!("ELF parseado com sucesso. Entry point: {:#x}", elf.entry);

        Ok(elf)
    }

    /// Calcula o intervalo de endereços virtuais do ELF
    ///
    /// # Argumentos
    /// * `elf` - Estrutura ELF parseada
    ///
    /// # Retorna
    /// (endereço_mínimo, endereço_máximo)
    pub fn calculate_address_range(elf: &Elf) -> (u64, u64) {
        let mut min_vaddr = u64::MAX;
        let mut max_vaddr = u64::MIN;

        for ph in elf.program_headers.iter() {
            if ph.p_type == goblin::elf::program_header::PT_LOAD {
                if ph.p_vaddr < min_vaddr {
                    min_vaddr = ph.p_vaddr;
                }
                let end_vaddr = ph.p_vaddr + ph.p_memsz;
                if end_vaddr > max_vaddr {
                    max_vaddr = end_vaddr;
                }
            }
        }

        (min_vaddr, max_vaddr)
    }
}
