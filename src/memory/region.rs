//! Definição de Regiões de Memória Física
//!
//! Este arquivo contém as estruturas primitivas que descrevem blocos de RAM.
//! É usado tanto pelo mapeador de memória (uefi) quanto pelo handoff para o
//! kernel.

/// Tipos de memória suportados pelo Bootloader.
///
/// Simplificamos a complexidade do UEFI (que tem dezenas de tipos)
/// para categorias que o Kernel realmente se importa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Garante layout C para compatibilidade com Kernel/ASM
pub enum MemoryRegionKind {
    /// Memória RAM livre e utilizável.
    Usable,
    /// Memória em uso pelo Bootloader (será reciclada pelo kernel depois).
    Bootloader,
    /// Código do Kernel ou Módulos carregados.
    Kernel,
    /// Memória reservada por hardware ou firmware (ACPI, BIOS, etc). NUNCA
    /// TOCAR.
    Reserved,
    /// Memória defeituosa detectada pelo POST.
    BadMemory,
}

/// Representa um intervalo contíguo de memória física.
///
/// # Design Industrial
/// Usamos `start` e `len` (em páginas) em vez de bytes para evitar
/// erros de alinhamento. O Bootloader opera sempre em granularidade de 4KiB.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PhysicalMemoryRegion {
    pub start:      u64,
    pub page_count: usize,
    pub kind:       MemoryRegionKind,
}

impl PhysicalMemoryRegion {
    /// Endereço de início em bytes
    pub fn start_addr(&self) -> u64 {
        self.start
    }

    /// Endereço de fim em bytes (exclusivo)
    pub fn end_addr(&self) -> u64 {
        self.start + (self.page_count as u64 * 4096)
    }

    /// Tamanho total em bytes
    pub fn size_in_bytes(&self) -> u64 {
        self.page_count as u64 * 4096
    }
}
