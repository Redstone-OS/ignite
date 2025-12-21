//! Abstração de Sistema Operacional (Ambiente de Execução)
//!
//! Define a interface que o módulo `arch` usa para interagir com o ambiente
//! subjacente (seja ele UEFI, BIOS ou Teste). Isso permite que o código de
//! baixo nível (paginação, gdt) seja agnóstico em relação ao firmware.

/// Tipo de memória alocada pelo OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsMemoryKind {
    /// Memória livre/usável.
    Free,
    /// Memória reservada pelo hardware ou firmware.
    Reserved,
    /// Memória reclamável (pode ser sobrescrita após o boot, ex: tabelas
    /// temporárias).
    Reclaim,
    /// Código do Bootloader ou Kernel.
    Code,
    /// Dados do Bootloader ou Kernel.
    Data,
}

/// Uma entrada no mapa de memória do OS.
#[derive(Debug, Clone, Copy)]
pub struct OsMemoryEntry {
    /// Endereço base físico.
    pub base: u64,
    /// Tamanho em bytes.
    pub size: u64,
    /// Tipo de memória.
    pub kind: OsMemoryKind,
}

/// Interface que o ambiente deve implementar.
pub trait Os {
    /// Aloca memória contígua, alinhada a página (4KiB) e preenchida com zeros.
    /// Retorna ponteiro físico ou null se falhar.
    fn alloc_zeroed_page_aligned(&self, size: usize) -> *mut u8;

    /// Mapeia memória (se o ambiente suportar paginação própria antes do
    /// kernel). Em UEFI, geralmente é no-op pois usamos identity map.
    fn map_memory(&self, phys: u64, virt: u64, size: u64, flags: u64);

    /// Registra uma região de memória usada.
    /// Útil para o bootloader rastrear o que ele mesmo alocou.
    fn add_memory_entry(&self, entry: OsMemoryEntry);
}

// Carrega a implementação UEFI se estivermos compilando para esse alvo.
#[cfg(target_os = "uefi")]
pub mod uefi;
