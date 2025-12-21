//! Validação de Cabeçalho ELF
//!
//! Este módulo garante que o binário carregado é seguro, íntegro e compatível
//! com a arquitetura alvo (x86_64) antes de qualquer tentativa de execução.
//!
//! # Segurança Industrial
//! Implementa verificações rigorosas de Magic Bytes, Arquitetura, Endianness e
//! Tipo de Arquivo para prevenir a execução de código corrompido ou malicioso.

// Alias 'elf_hdr' evita colisão de nomes com a variável 'header'
use goblin::elf::header as elf_hdr;

use crate::core::error::{BootError, ElfError, Result};

/// Verifica se o cabeçalho ELF é válido e compatível com este Bootloader.
///
/// # Argumentos
/// * `header` - Referência ao cabeçalho ELF já parseado pela crate `goblin`.
///
/// # Retorna
/// * `Ok(())` se o arquivo for seguro para carregar.
/// * `Err(BootError::Elf(...))` descrevendo a violação específica.
pub fn validate_header(header: &elf_hdr::Header) -> Result<()> {
    // 1. Verificar Magic Bytes (0x7F 'E' 'L' 'F')
    // Usamos bytes literais para evitar erros de importação de constantes
    if header.e_ident[0] != 0x7F
        || header.e_ident[1] != b'E'
        || header.e_ident[2] != b'L'
        || header.e_ident[3] != b'F'
    {
        return Err(BootError::Elf(ElfError::InvalidMagic));
    }

    // 2. Verificar Classe/Arquitetura (Deve ser 64-bit)
    // O Ignite (v0.4.0) roda exclusivamente em Long Mode (64-bit).
    // TODO(Arch): Adicionar suporte condicional para 32-bit se portarmos para x86
    // legado.
    if header.e_ident[elf_hdr::EI_CLASS] != elf_hdr::ELFCLASS64 {
        return Err(BootError::Elf(ElfError::InvalidArchitecture));
    }

    // 3. Verificar Endianness (Deve ser Little Endian para x86_64)
    // x86_64 é sempre Little Endian.
    // TODO(Arch): Ao suportar AArch64 ou RISC-V, verificar se Big Endian é
    // necessário.
    if header.e_ident[elf_hdr::EI_DATA] != elf_hdr::ELFDATA2LSB {
        return Err(BootError::Elf(ElfError::InvalidEndianness));
    }

    // 4. Verificar Machine Type (ISA Alvo)
    // Garante que não estamos tentando rodar código ARM em uma CPU Intel/AMD.
    // TODO(Arch): Expandir match para EM_AARCH64 e EM_RISCV quando compilado para
    // esses alvos.
    if header.e_machine != elf_hdr::EM_X86_64 {
        return Err(BootError::Elf(ElfError::InvalidMachine));
    }

    // 5. Verificar Tipo de Arquivo (Executável ou Shared Object/PIE)
    // ET_EXEC = 2, ET_DYN = 3
    if header.e_type != elf_hdr::ET_EXEC && header.e_type != elf_hdr::ET_DYN {
        return Err(BootError::Elf(ElfError::UnsupportedFileType));
    }

    Ok(())
}
