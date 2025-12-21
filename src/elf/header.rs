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
    // Garante que o arquivo é realmente um binário ELF e não texto ou lixo.
    // Usamos o alias `elf_hdr` para acessar as constantes do módulo.
    let magic_valid = header.e_ident[elf_hdr::EI_MAG0] == elf_hdr::ELFMAG[0]
        && header.e_ident[elf_hdr::EI_MAG1] == elf_hdr::ELFMAG[1]
        && header.e_ident[elf_hdr::EI_MAG2] == elf_hdr::ELFMAG[2]
        && header.e_ident[elf_hdr::EI_MAG3] == elf_hdr::ELFMAG[3];

    if !magic_valid {
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

    // 5. Verificar Versão do ELF
    // Deve ser a versão atual (1). Versões futuras podem mudar o layout.
    if header.e_ident[elf_hdr::EI_VERSION] != elf_hdr::EV_CURRENT {
        // Warning: Versão ELF desconhecida, mas tentamos prosseguir se o parser
        // aceitou. Em ambientes de alta segurança, isso deveria
        // retornar erro.
    }

    // 6. Verificar Tipo de Arquivo
    // Aceitamos apenas:
    // - ET_EXEC: Executável estático (com endereços absolutos).
    // - ET_DYN: Executável dinâmico/PIE (Position Independent Executable - Kernels
    //   modernos com KASLR).
    match header.e_type {
        elf_hdr::ET_EXEC | elf_hdr::ET_DYN => {},
        _ => return Err(BootError::Elf(ElfError::UnsupportedFileType)),
    }

    // 7. Validação de Ponto de Entrada (Entry Point)
    // Um entry point 0x0 em um executável estático geralmente indica erro de
    // linkagem. Em PIE (DYN), 0x0 pode ser um offset válido, mas é raro e
    // suspeito para um kernel.
    if header.e_type == elf_hdr::ET_EXEC && header.e_entry == 0 {
        return Err(BootError::Elf(ElfError::InvalidEntryPoint));
    }

    Ok(())
}
