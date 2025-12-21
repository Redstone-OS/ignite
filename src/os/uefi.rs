//! Implementação da Trait OS para UEFI
//!
//! Conecta as necessidades do `arch` (alocação de páginas) aos serviços
//! `src/uefi`. Utiliza `BootServices` para alocar memória física real.

use super::{Os, OsMemoryEntry};
use crate::uefi::{
    system_table,
    table::boot::{AllocateType, MemoryType},
};

/// Driver do ambiente UEFI.
pub struct UefiOs;

impl Os for UefiOs {
    fn alloc_zeroed_page_aligned(&self, size: usize) -> *mut u8 {
        let bs = system_table().boot_services();

        // Calcular número de páginas (4KiB = 0x1000)
        // (size + 4095) / 4096 faz o arredondamento para cima (ceil)
        let pages = (size + 0xFFF) / 0x1000;

        // Alocar usando UEFI Boot Services
        // Usamos `LoaderData` para indicar que esta memória contém dados do bootloader
        // que o kernel pode, eventualmente, reclamar ou preservar.
        match bs.allocate_pages(
            AllocateType::AllocateAnyPages,
            MemoryType::LoaderData,
            pages,
        ) {
            Ok(addr) => {
                let ptr = addr as *mut u8;

                // Segurança: Zerar a memória para evitar vazamento de dados antigos
                // e garantir estado determinístico para tabelas de página.
                unsafe {
                    core::ptr::write_bytes(ptr, 0, pages * 0x1000);
                }

                ptr
            },
            Err(_) => {
                // Em caso de falha (OOM), retornamos null.
                // O chamador (arch) deve lidar com isso.
                core::ptr::null_mut()
            },
        }
    }

    fn map_memory(&self, _phys: u64, _virt: u64, _size: u64, _flags: u64) {
        // UEFI roda em identity map (Endereço Físico == Endereço Virtual) na
        // maior parte do tempo. O módulo `arch` usa suas próprias
        // funções para configurar as tabelas de página do KERNEL.
        // Portanto, não precisamos alterar o mapeamento ativo do UEFI aqui.
    }

    fn add_memory_entry(&self, _entry: OsMemoryEntry) {
        // Em UEFI, o mapa de memória é gerenciado nativamente pelo firmware.
        // Quando chamamos `allocate_pages`, o firmware já atualiza seu mapa
        // interno. Não precisamos manter um mapa paralelo manual aqui
        // para o UEFI.
    }
}
