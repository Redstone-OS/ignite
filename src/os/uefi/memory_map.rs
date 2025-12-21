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
        // Implementação manual para evitar alocação durante exit
        // Baseado em: https://github.com/rust-osdev/uefi-rs/blob/main/uefi/src/boot.rs#L1285-1391

        unsafe {
            // Obter system table
            let st = uefi::table::system_table_boot().unwrap();

            // Buffer grande na stack - não usa heap allocator!
            const BUFFER_SIZE: usize = 16384; // 16KB deve ser suficiente
            let mut buffer = [0u8; BUFFER_SIZE];

            // Loop com até 2 tentativas (padrão do Linux kernel)
            for attempt in 0..2 {
                // Obter memory map
                let mut mmap_size = BUFFER_SIZE;
                let mut mmap_key = 0usize;
                let mut desc_size = 0usize;
                let mut desc_version = 0u32;

                // Chamar GetMemoryMap diretamente via ponteiro da tabela
                let boot_services = st.boot_services();
                let bs_ptr = boot_services as *const _ as *const uefi::table::boot::BootServices;

                // Acessar a tabela raw através de transmute (unsafe mas necessário)
                let raw_table: &uefi_raw::table::boot::BootServices = core::mem::transmute(bs_ptr);

                let status = (raw_table.get_memory_map)(
                    &mut mmap_size,
                    buffer.as_mut_ptr() as *mut uefi_raw::table::boot::MemoryDescriptor,
                    &mut mmap_key,
                    &mut desc_size,
                    &mut desc_version,
                );

                // Se GetMemoryMap falhou, desistir
                if !uefi::Status::from(status).is_success() {
                    if attempt == 0 {
                        continue; // Tentar novamente
                    }
                    break; // Desistir após 2 tentativas
                }

                // Chamar ExitBootServices com o map_key obtido
                let exit_status =
                    (raw_table.exit_boot_services)(uefi::boot::image_handle().as_ptr(), mmap_key);

                // Se sucesso, terminar
                if uefi::Status::from(exit_status).is_success() {
                    break;
                }

                // Se falhou (map_key inválido), loop vai tentar novamente
            }
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
