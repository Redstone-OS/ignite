use alloc::{vec, vec::Vec};
use core::{mem, ptr};

use uefi::table::boot::{MemoryDescriptor, MemoryType};

use crate::os::{OsMemoryEntry, OsMemoryKind};

pub struct MemoryMapIter {
    // Campos stub
}

impl MemoryMapIter {
    pub fn new() -> Self {
        // Stub
        Self {}
    }

    pub fn exit_boot_services(self) {
        // Stub
        // panic!("exit_boot_services stub");
        // Uso de panic no bootloader pode duplicar o pânico.
        loop {}
    }
}

impl Iterator for MemoryMapIter {
    type Item = OsMemoryEntry;
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub unsafe fn memory_map() -> MemoryMapIter {
    MemoryMapIter::new()
}
