//! Validação de Cabeçalho ELF
//!
//! Garante que o binário é seguro e compatível com a arquitetura x86_64
//! antes de tentarmos carregá-lo.

use goblin::elf::header;

use crate::core::error::{BootError, ElfError, Result};

/// Verifica se o cabeçalho ELF é compatível com este Bootloader (x86_64/UEFI).
pub fn validate_header(header: &header::Header) -> Result<()> {
    // 1. Verificar Magic Bytes (0x7F 'E' 'L' 'F')
    if header.e_ident[header::EI_MAG0] != header::ELFMAG0
        || header.e_ident[header::EI_MAG1] != header::ELFMAG1
        || header.e_ident[header::EI_MAG2] != header::ELFMAG2
        || header.e_ident[header::EI_MAG3] != header::ELFMAG3
    {
        return Err(BootError::Elf(ElfError::InvalidMagic));
    }

    // 2. Verificar Arquitetura (Deve ser 64-bit)
    if header.e_ident[header::EI_CLASS] != header::ELFCLASS64 {
        return Err(BootError::Elf(ElfError::InvalidArchitecture));
    }

    // 3. Verificar Endianness (Deve ser Little Endian para x86_64)
    if header.e_ident[header::EI_DATA] != header::ELFDATA2LSB {
        return Err(BootError::Elf(ElfError::InvalidEndianness));
    }

    // 4. Verificar Machine Type (Deve ser x86-64 / AMD64)
    if header.e_machine != header::EM_X86_64 {
        return Err(BootError::Elf(ElfError::InvalidMachine));
    }

    // 5. Verificar Tipo de Arquivo (Executável ou Shared Object/PIE)
    if header.e_type != header::ET_EXEC && header.e_type != header::ET_DYN {
        return Err(BootError::Elf(ElfError::UnsupportedFileType));
    }

    Ok(())
}
