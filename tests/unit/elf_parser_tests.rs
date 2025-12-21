//! Testes Unitários para o Parser ELF
//!
//! Este módulo testa o parser de arquivos ELF64 do bootloader Ignite.
//! Valida a correta leitura, validação e processamento de executáveis ELF.

#![cfg(test)]

use goblin::elf::{
    Elf,
    header::{ELFMAG, ET_EXEC, header64},
    program_header::{PT_LOAD, ProgramHeader},
};

/// Cria um cabeçalho ELF64 válido minimalista
fn criar_cabecalho_elf_valido() -> [u8; 64] {
    let mut header = [0u8; 64];

    // Magic number ELF
    header[0..4].copy_from_slice(&ELFMAG);

    // EI_CLASS: 64-bit
    header[4] = 2; // ELFCLASS64

    // EI_DATA: Little endian
    header[5] = 1; // ELFDATA2LSB

    // EI_VERSION: Current version
    header[6] = 1;

    // e_type: Executável
    header[16] = ET_EXEC as u8;
    header[17] = 0;

    // e_machine: x86_64
    header[18] = 0x3E;
    header[19] = 0;

    // e_version
    header[20] = 1;

    // e_entry: Entry point em 0x100000 (exemplo)
    let entry: u64 = 0x100000;
    header[24..32].copy_from_slice(&entry.to_le_bytes());

    // e_phoff: Program header offset em 64
    let phoff: u64 = 64;
    header[32..40].copy_from_slice(&phoff.to_le_bytes());

    // e_shoff: Section header offset (0 = sem sections)
    header[40..48].copy_from_slice(&[0u8; 8]);

    // e_ehsize: Tamanho do ELF header (64 bytes)
    header[52..54].copy_from_slice(&64u16.to_le_bytes());

    // e_phentsize: Tamanho de cada program header (56 bytes)
    header[54..56].copy_from_slice(&56u16.to_le_bytes());

    // e_phnum: Número de program headers (1)
    header[56..58].copy_from_slice(&1u16.to_le_bytes());

    header
}

/// Cria um program header PT_LOAD válido
fn criar_program_header_load(vaddr: u64, memsz: u64, filesz: u64) -> [u8; 56] {
    let mut ph = [0u8; 56];

    // p_type: PT_LOAD
    ph[0..4].copy_from_slice(&(PT_LOAD as u32).to_le_bytes());

    // p_flags: R+W+X (7)
    ph[4..8].copy_from_slice(&7u32.to_le_bytes());

    // p_offset
    ph[8..16].copy_from_slice(&128u64.to_le_bytes());

    // p_vaddr
    ph[16..24].copy_from_slice(&vaddr.to_le_bytes());

    // p_paddr (mesmo que vaddr)
    ph[24..32].copy_from_slice(&vaddr.to_le_bytes());

    // p_filesz
    ph[32..40].copy_from_slice(&filesz.to_le_bytes());

    // p_memsz
    ph[40..48].copy_from_slice(&memsz.to_le_bytes());

    // p_align
    ph[48..56].copy_from_slice(&4096u64.to_le_bytes());

    ph
}

/// Cria um arquivo ELF64 válido completo
fn criar_elf_valido() -> Vec<u8> {
    let mut elf = Vec::new();

    // Adicionar cabeçalho ELF
    elf.extend_from_slice(&criar_cabecalho_elf_valido());

    // Adicionar program header
    elf.extend_from_slice(&criar_program_header_load(0x100000, 0x1000, 0x1000));

    // Adicionar dados dummy (mínimo necessário)
    elf.resize(128 + 0x1000, 0);

    elf
}

#[test]
fn test_parse_elf_valido() {
    // Arrange: Criar um ELF válido
    let elf_data = criar_elf_valido();

    // Act: Parsear o ELF
    let resultado = Elf::parse(&elf_data);

    // Assert: Deve parsear com sucesso
    assert!(resultado.is_ok(), "Falha ao parsear ELF válido");

    let elf = resultado.unwrap();
    assert_eq!(elf.entry, 0x100000, "Entry point incorreto");
    assert!(elf.is_64, "Deveria ser ELF de 64 bits");
}

#[test]
fn test_elf_valida_entry_point() {
    // Arrange: Criar ELF válido
    let elf_data = criar_elf_valido();
    let elf = Elf::parse(&elf_data).unwrap();

    // Assert: Entry point não pode ser zero
    assert_ne!(elf.entry, 0, "Entry point não deve ser zero");
}

#[test]
fn test_elf_detecta_segmentos_load() {
    // Arrange: Criar ELF válido com PT_LOAD
    let elf_data = criar_elf_valido();
    let elf = Elf::parse(&elf_data).unwrap();

    // Act: Procurar por segmentos PT_LOAD
    let tem_load = elf.program_headers.iter().any(|ph| ph.p_type == PT_LOAD);

    // Assert: Deve ter pelo menos um segmento PT_LOAD
    assert!(tem_load, "ELF deve ter pelo menos um segmento PT_LOAD");
}

#[test]
fn test_elf_calcula_range_enderecos() {
    // Arrange: Criar ELF válido
    let elf_data = criar_elf_valido();
    let elf = Elf::parse(&elf_data).unwrap();

    // Act: Calcular range de endereços
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr = u64::MIN;

    for ph in elf.program_headers.iter() {
        if ph.p_type == PT_LOAD {
            if ph.p_vaddr < min_vaddr {
                min_vaddr = ph.p_vaddr;
            }
            let end_vaddr = ph.p_vaddr + ph.p_memsz;
            if end_vaddr > max_vaddr {
                max_vaddr = end_vaddr;
            }
        }
    }

    // Assert: Range deve estar correto
    assert_eq!(min_vaddr, 0x100000, "Endereço mínimo incorreto");
    assert_eq!(max_vaddr, 0x101000, "Endereço máximo incorreto");
}

#[test]
fn test_rejeita_arquivo_invalido() {
    // Arrange: Dados inválidos
    let dados_invalidos = vec![0u8; 100];

    // Act: Tentar parsear
    let resultado = Elf::parse(&dados_invalidos);

    // Assert: Deve falhar
    assert!(resultado.is_err(), "Deveria rejeitar arquivo inválido");
}

#[test]
fn test_rejeita_magic_number_invalido() {
    // Arrange: Cabeçalho com magic number errado
    let mut dados = criar_cabecalho_elf_valido();
    dados[0] = 0xFF; // Corromper magic number

    // Act: Tentar parsear
    let resultado = Elf::parse(&dados);

    // Assert: Deve falhar
    assert!(resultado.is_err(), "Deveria rejeitar magic number inválido");
}

#[test]
fn test_elf_sem_entry_point() {
    // Arrange: Criar cabeçalho com entry point = 0
    let mut header = criar_cabecalho_elf_valido();
    header[24..32].copy_from_slice(&0u64.to_le_bytes()); // Zerar entry point

    let mut elf_data = Vec::new();
    elf_data.extend_from_slice(&header);
    elf_data.extend_from_slice(&criar_program_header_load(0x100000, 0x1000, 0x1000));
    elf_data.resize(128 + 0x1000, 0);

    // Act: Parsear
    let elf = Elf::parse(&elf_data).unwrap();

    // Assert: Entry point deve ser zero (inválido para nossos propósitos)
    assert_eq!(elf.entry, 0, "Entry point deveria ser zero");
}

#[test]
fn test_elf_multiplos_segmentos_load() {
    // Arrange: Criar ELF com múltiplos segmentos PT_LOAD
    let mut elf = Vec::new();

    let mut header = criar_cabecalho_elf_valido();
    // Modificar para 2 program headers
    header[56..58].copy_from_slice(&2u16.to_le_bytes());

    elf.extend_from_slice(&header);
    elf.extend_from_slice(&criar_program_header_load(0x100000, 0x1000, 0x1000));
    elf.extend_from_slice(&criar_program_header_load(0x200000, 0x2000, 0x2000));
    elf.resize(128 + 0x3000, 0);

    // Act: Parsear
    let elf_parsed = Elf::parse(&elf).unwrap();

    // Assert: Deve ter 2 segmentos PT_LOAD
    let count_load = elf_parsed
        .program_headers
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD)
        .count();

    assert_eq!(count_load, 2, "Deveria ter 2 segmentos PT_LOAD");
}

#[test]
fn test_elf_arquivo_vazio() {
    // Arrange: Arquivo vazio
    let dados_vazios: Vec<u8> = Vec::new();

    // Act: Tentar parsear
    let resultado = Elf::parse(&dados_vazios);

    // Assert: Deve falhar
    assert!(resultado.is_err(), "Deveria rejeitar arquivo vazio");
}

#[test]
fn test_elf_tamanho_minimo() {
    // Arrange: Arquivo muito pequeno (menor que header ELF)
    let dados_pequenos = vec![0x7F, b'E', b'L', b'F']; // Apenas magic, sem resto

    // Act: Tentar parsear
    let resultado = Elf::parse(&dados_pequenos);

    // Assert: Deve falhar
    assert!(resultado.is_err(), "Deveria rejeitar arquivo muito pequeno");
}
