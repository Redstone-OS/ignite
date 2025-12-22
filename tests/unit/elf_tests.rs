//! Testes Unitários para o módulo ELF
//!
//! Testa parsing e validação de binários ELF64.

#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;

/// Testa validação de magic bytes ELF
#[test]
fn test_elf_magic() {
    const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

    fn is_elf(data: &[u8]) -> bool {
        data.len() >= 4 && data[0..4] == ELF_MAGIC
    }

    assert!(is_elf(&[0x7F, b'E', b'L', b'F', 0, 0, 0, 0]));
    assert!(!is_elf(&[0x00, 0x00, 0x00, 0x00]));
    assert!(!is_elf(&[0x7F, b'E', b'L'])); // Muito curto
}

/// Testa validação de classe ELF (32/64 bit)
#[test]
fn test_elf_class() {
    const EI_CLASS: usize = 4;
    const ELFCLASS32: u8 = 1;
    const ELFCLASS64: u8 = 2;

    let elf64_header = [0x7F, b'E', b'L', b'F', ELFCLASS64, 0, 0, 0];
    let elf32_header = [0x7F, b'E', b'L', b'F', ELFCLASS32, 0, 0, 0];

    assert_eq!(elf64_header[EI_CLASS], ELFCLASS64);
    assert_eq!(elf32_header[EI_CLASS], ELFCLASS32);
}

/// Testa validação de endianness
#[test]
fn test_elf_endianness() {
    const EI_DATA: usize = 5;
    const ELFDATA2LSB: u8 = 1; // Little endian
    const ELFDATA2MSB: u8 = 2; // Big endian

    let le_header = [0x7F, b'E', b'L', b'F', 2, ELFDATA2LSB, 0, 0];
    let be_header = [0x7F, b'E', b'L', b'F', 2, ELFDATA2MSB, 0, 0];

    assert_eq!(le_header[EI_DATA], ELFDATA2LSB);
    assert_eq!(be_header[EI_DATA], ELFDATA2MSB);
}

/// Testa parsing de e_type (executable, shared, etc)
#[test]
fn test_elf_type() {
    #[derive(Debug, PartialEq)]
    enum ElfType {
        None,
        Relocatable,
        Executable,
        Shared,
        Core,
        Unknown(u16),
    }

    fn parse_elf_type(et_type: u16) -> ElfType {
        match et_type {
            0 => ElfType::None,
            1 => ElfType::Relocatable,
            2 => ElfType::Executable,
            3 => ElfType::Shared,
            4 => ElfType::Core,
            n => ElfType::Unknown(n),
        }
    }

    assert_eq!(parse_elf_type(0), ElfType::None);
    assert_eq!(parse_elf_type(2), ElfType::Executable);
    assert_eq!(parse_elf_type(3), ElfType::Shared);
    assert_eq!(parse_elf_type(999), ElfType::Unknown(999));
}

/// Testa validação de machine type
#[test]
fn test_elf_machine() {
    const EM_X86_64: u16 = 0x3E;
    const EM_AARCH64: u16 = 0xB7;
    const EM_RISCV: u16 = 0xF3;

    fn is_supported_arch(e_machine: u16) -> bool {
        matches!(e_machine, EM_X86_64 | EM_AARCH64 | EM_RISCV)
    }

    assert!(is_supported_arch(EM_X86_64));
    assert!(is_supported_arch(EM_AARCH64));
    assert!(is_supported_arch(EM_RISCV));
    assert!(!is_supported_arch(0x00)); // Nenhuma máquina
}

/// Testa parsing de program header type
#[test]
fn test_program_header_type() {
    const PT_NULL: u32 = 0;
    const PT_LOAD: u32 = 1;
    const PT_DYNAMIC: u32 = 2;
    const PT_INTERP: u32 = 3;
    const PT_NOTE: u32 = 4;
    const PT_PHDR: u32 = 6;
    const PT_TLS: u32 = 7;

    fn is_loadable(p_type: u32) -> bool {
        p_type == PT_LOAD
    }

    assert!(is_loadable(PT_LOAD));
    assert!(!is_loadable(PT_NULL));
    assert!(!is_loadable(PT_DYNAMIC));
}

/// Testa flags de program header
#[test]
fn test_program_header_flags() {
    const PF_X: u32 = 1 << 0; // Executable
    const PF_W: u32 = 1 << 1; // Writable
    const PF_R: u32 = 1 << 2; // Readable

    fn is_executable(flags: u32) -> bool {
        (flags & PF_X) != 0
    }

    fn is_writable(flags: u32) -> bool {
        (flags & PF_W) != 0
    }

    fn is_readable(flags: u32) -> bool {
        (flags & PF_R) != 0
    }

    // Code segment: R-X
    let code_flags = PF_R | PF_X;
    assert!(is_readable(code_flags));
    assert!(is_executable(code_flags));
    assert!(!is_writable(code_flags));

    // Data segment: RW-
    let data_flags = PF_R | PF_W;
    assert!(is_readable(data_flags));
    assert!(is_writable(data_flags));
    assert!(!is_executable(data_flags));
}

/// Testa cálculo de offset de file para segment
#[test]
fn test_file_offset_calculation() {
    struct ProgramHeader {
        p_offset: u64,
        p_filesz: u64,
        p_memsz:  u64,
    }

    impl ProgramHeader {
        fn file_range(&self) -> (u64, u64) {
            (self.p_offset, self.p_offset + self.p_filesz)
        }

        fn has_bss(&self) -> bool {
            self.p_memsz > self.p_filesz
        }

        fn bss_size(&self) -> u64 {
            if self.has_bss() {
                self.p_memsz - self.p_filesz
            } else {
                0
            }
        }
    }

    let segment = ProgramHeader {
        p_offset: 0x1000,
        p_filesz: 0x2000,
        p_memsz:  0x3000, // 0x1000 bytes de BSS
    };

    let (start, end) = segment.file_range();
    assert_eq!(start, 0x1000);
    assert_eq!(end, 0x3000);

    assert!(segment.has_bss());
    assert_eq!(segment.bss_size(), 0x1000);
}

/// Testa validação de entry point
#[test]
fn test_entry_point_validation() {
    fn is_valid_entry(entry: u64) -> bool {
        entry != 0 && entry % 2 == 0 // Alinhado (pelo menos)
    }

    assert!(is_valid_entry(0x1000));
    assert!(is_valid_entry(0x400000));
    assert!(!is_valid_entry(0x0)); // Null
    assert!(!is_valid_entry(0x1001)); // Não alinhado
}

/// Testa validação de alinhamento de segment
#[test]
fn test_segment_alignment() {
    fn is_aligned(addr: u64, align: u64) -> bool {
        align == 0 || addr % align == 0
    }

    struct Segment {
        p_vaddr: u64,
        p_align: u64,
    }

    impl Segment {
        fn is_properly_aligned(&self) -> bool {
            is_aligned(self.p_vaddr, self.p_align)
        }
    }

    let aligned_seg = Segment {
        p_vaddr: 0x1000,
        p_align: 0x1000,
    };
    assert!(aligned_seg.is_properly_aligned());

    let misaligned_seg = Segment {
        p_vaddr: 0x1001,
        p_align: 0x1000,
    };
    assert!(!misaligned_seg.is_properly_aligned());
}

/// Testa parsing de section header
#[test]
fn test_section_header() {
    #[derive(Debug, PartialEq)]
    enum SectionType {
        Null,
        ProgBits,
        SymTab,
        StrTab,
        Rela,
        NoBits,
        Unknown(u32),
    }

    fn parse_section_type(sh_type: u32) -> SectionType {
        match sh_type {
            0 => SectionType::Null,
            1 => SectionType::ProgBits,
            2 => SectionType::SymTab,
            3 => SectionType::StrTab,
            4 => SectionType::Rela,
            8 => SectionType::NoBits,
            n => SectionType::Unknown(n),
        }
    }

    assert_eq!(parse_section_type(0), SectionType::Null);
    assert_eq!(parse_section_type(1), SectionType::ProgBits);
    assert_eq!(parse_section_type(8), SectionType::NoBits);
}

/// Testa cálculo de tamanho total de carregamento
#[test]
fn test_load_size_calculation() {
    struct LoadSegment {
        p_vaddr: u64,
        p_memsz: u64,
    }

    fn calculate_load_size(segments: &[LoadSegment]) -> u64 {
        if segments.is_empty() {
            return 0;
        }

        let min_addr = segments.iter().map(|s| s.p_vaddr).min().unwrap();
        let max_addr = segments
            .iter()
            .map(|s| s.p_vaddr + s.p_memsz)
            .max()
            .unwrap();

        max_addr - min_addr
    }

    let segments = vec![
        LoadSegment {
            p_vaddr: 0x1000,
            p_memsz: 0x1000,
        },
        LoadSegment {
            p_vaddr: 0x3000,
            p_memsz: 0x2000,
        },
    ];

    let total_size = calculate_load_size(&segments);
    assert_eq!(total_size, 0x4000); // 0x5000 - 0x1000
}

/// Testa validação de string table
#[test]
fn test_string_table_validation() {
    fn get_string(strtab: &[u8], offset: usize) -> Option<&str> {
        if offset >= strtab.len() {
            return None;
        }

        // Encontrar null terminator
        let end = strtab[offset..].iter().position(|&b| b == 0)?;

        core::str::from_utf8(&strtab[offset..offset + end]).ok()
    }

    let strtab = b"\0.text\0.data\0.bss\0";

    assert_eq!(get_string(strtab, 0), Some(""));
    assert_eq!(get_string(strtab, 1), Some(".text"));
    assert_eq!(get_string(strtab, 7), Some(".data"));
    assert_eq!(get_string(strtab, 13), Some(".bss"));
    assert_eq!(get_string(strtab, 100), None); // Out of bounds
}

/// Testa conversão little-endian
#[test]
fn test_little_endian_conversion() {
    fn read_u16_le(bytes: &[u8]) -> u16 {
        u16::from_le_bytes([bytes[0], bytes[1]])
    }

    fn read_u32_le(bytes: &[u8]) -> u32 {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn read_u64_le(bytes: &[u8]) -> u64 {
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    assert_eq!(read_u16_le(&[0x34, 0x12]), 0x1234);
    assert_eq!(read_u32_le(&[0x78, 0x56, 0x34, 0x12]), 0x12345678);
    assert_eq!(
        read_u64_le(&[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12]),
        0x1234567890ABCDEF
    );
}
