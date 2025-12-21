//! Abstração e Sanitização do Mapa de Memória
//!
//! O UEFI retorna um mapa fragmentado. A função deste módulo é:
//! 1. Iterar sobre o mapa cru do UEFI.
//! 2. Traduzir tipos UEFI -> MemoryRegionKind (nosso tipo).
//! 3. Ignorar regiões irrelevantes.

use super::region::{MemoryRegionKind, PhysicalMemoryRegion};
use crate::uefi::table::boot::{MemoryDescriptor, MemoryType};

/// Iterador que converte descritores UEFI em Regiões Físicas Limpas.
///
/// Isso desacopla o resto do sistema da API do UEFI. Se mudarmos para
/// BIOS/Multiboot2, só este arquivo muda.
pub struct MemoryMapIter<'a> {
    descriptors: core::slice::Iter<'a, MemoryDescriptor>,
}

impl<'a> MemoryMapIter<'a> {
    pub fn new(descriptors: &'a [MemoryDescriptor]) -> Self {
        Self {
            descriptors: descriptors.iter(),
        }
    }
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = PhysicalMemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        // Itera até achar um descritor válido que queiramos expor
        for desc in self.descriptors.by_ref() {
            let kind = match desc.r#type {
                MemoryType::ConventionalMemory => MemoryRegionKind::Usable,
                MemoryType::LoaderCode | MemoryType::LoaderData => MemoryRegionKind::Bootloader,
                MemoryType::ACPIReclaimMemory | MemoryType::ACPIMemoryNVS => {
                    MemoryRegionKind::Reserved
                },
                MemoryType::RuntimeServicesCode | MemoryType::RuntimeServicesData => {
                    MemoryRegionKind::Reserved
                },
                // Mapeamos o resto como reservado por segurança
                _ => MemoryRegionKind::Reserved,
            };

            // Otimização: Ignorar regiões vazias ou nulas que alguns firmwares bugados
            // reportam
            if desc.page_count == 0 {
                continue;
            }

            return Some(PhysicalMemoryRegion {
                start: desc.physical_start,
                page_count: desc.page_count as usize,
                kind,
            });
        }
        None
    }
}
