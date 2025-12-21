//! Abstração e Sanitização do Mapa de Memória

use super::region::{MemoryRegionKind, PhysicalMemoryRegion};
use crate::uefi::table::boot::{MemoryDescriptor, MemoryType};

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
        for desc in self.descriptors.by_ref() {
            // CORREÇÃO: desc.ty em vez de desc.r#type
            // CORREÇÃO: MemoryType::from(u32) ou match manual se o enum não tiver From
            // Assumindo que MemoryType é enum u32 compatível:
            let mt = unsafe { core::mem::transmute::<u32, MemoryType>(desc.ty) };

            let kind = match mt {
                MemoryType::ConventionalMemory => MemoryRegionKind::Usable,
                MemoryType::LoaderCode | MemoryType::LoaderData => MemoryRegionKind::Bootloader,
                MemoryType::ACPIReclaimMemory | MemoryType::ACPIMemoryNVS => {
                    MemoryRegionKind::Reserved
                },
                _ => MemoryRegionKind::Reserved,
            };

            // CORREÇÃO: desc.number_of_pages em vez de desc.page_count
            if desc.number_of_pages == 0 {
                continue;
            }

            return Some(PhysicalMemoryRegion {
                start: desc.physical_start,
                page_count: desc.number_of_pages as usize,
                kind,
            });
        }
        None
    }
}
