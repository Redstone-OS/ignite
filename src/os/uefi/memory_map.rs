use uefi::{mem::memory_map::MemoryMap, table::boot::MemoryType};

use crate::os::{OsMemoryEntry, OsMemoryKind};

pub struct MemoryMapIter {
    entries: alloc::vec::Vec<OsMemoryEntry>,
    index:   usize,
}

impl MemoryMapIter {
    pub fn new() -> Self {
        let mut entries = alloc::vec::Vec::new();

        // Usar a nova API freestanding do uefi 0.31
        let mmap =
            uefi::boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");

        for desc in mmap.entries() {
            let kind = match desc.ty {
                MemoryType::CONVENTIONAL => OsMemoryKind::Free,
                MemoryType::LOADER_CODE | MemoryType::LOADER_DATA => OsMemoryKind::Reclaim,
                MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA => {
                    OsMemoryKind::Reclaim
                },
                MemoryType::RUNTIME_SERVICES_CODE | MemoryType::RUNTIME_SERVICES_DATA => {
                    OsMemoryKind::Reserved
                },
                MemoryType::ACPI_RECLAIM => OsMemoryKind::Reclaim,
                _ => OsMemoryKind::Reserved,
            };

            entries.push(OsMemoryEntry {
                base: desc.phys_start,
                size: desc.page_count * 4096,
                kind,
            });
        }

        Self { entries, index: 0 }
    }

    pub fn exit_boot_services(self) {
        // SOLUÇÃO FINAL: Usar função freestanding de uefi-rs 0.31
        // Ela aloca internamente mas é a ÚNICA maneira segura
        // A tentativa de usar raw API causava Invalid Opcode
        unsafe {
            let _mmap_owned = uefi::boot::exit_boot_services(MemoryType::LOADER_DATA);
            // Ignoramos o retorno - boot services estão desligados agora
        }
    }
}

impl Iterator for MemoryMapIter {
    type Item = OsMemoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let entry = self.entries[self.index];
            self.index += 1;
            Some(entry)
        } else {
            None
        }
    }
}

pub unsafe fn memory_map() -> MemoryMapIter {
    MemoryMapIter::new()
}
