#![allow(unaligned_references)]

//! Native FAT32 Filesystem Driver
//!
//! Reads FAT12/16/32 filesystems without relying on UEFI

use alloc::{string::String, vec::Vec};

use crate::error::{BootError, Result};

/// BPB (BIOS Parameter Block) for FAT
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct BiosParameterBlock {
    jmp_boot:            [u8; 3],
    oem_name:            [u8; 8],
    bytes_per_sector:    u16,
    sectors_per_cluster: u8,
    reserved_sectors:    u16,
    num_fats:            u8,
    root_entry_count:    u16,
    total_sectors_16:    u16,
    media:               u8,
    fat_size_16:         u16,
    sectors_per_track:   u16,
    num_heads:           u16,
    hidden_sectors:      u32,
    total_sectors_32:    u32,
}

/// Extended BPB for FAT32
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Fat32ExtendedBPB {
    fat_size_32:        u32,
    ext_flags:          u16,
    fs_version:         u16,
    root_cluster:       u32,
    fs_info:            u16,
    backup_boot_sector: u16,
    reserved:           [u8; 12],
    drive_number:       u8,
    reserved1:          u8,
    boot_signature:     u8,
    volume_id:          u32,
    volume_label:       [u8; 11],
    fs_type:            [u8; 8],
}

/// FAT directory entry
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DirectoryEntry {
    name:              [u8; 11],
    attr:              u8,
    reserved:          u8,
    create_time_tenth: u8,
    create_time:       u16,
    create_date:       u16,
    access_date:       u16,
    first_cluster_hi:  u16,
    write_time:        u16,
    write_date:        u16,
    first_cluster_lo:  u16,
    file_size:         u32,
}

/// FAT filesystem implementation
pub struct Fat32FileSystem {
    bytes_per_sector:    u16,
    sectors_per_cluster: u8,
    reserved_sectors:    u16,
    num_fats:            u8,
    fat_size:            u32,
    root_cluster:        u32,
    total_sectors:       u32,
}

impl Fat32FileSystem {
    /// Mount a FAT filesystem from boot sector
    pub fn mount(boot_sector: &[u8]) -> Result<Self> {
        if boot_sector.len() < 512 {
            return Err(BootError::Generic("Boot sector too small"));
        }

        // Parse BPB
        let bpb = unsafe { &*(boot_sector.as_ptr() as *const BiosParameterBlock) };

        // Validate signature
        if boot_sector[510] != 0x55 || boot_sector[511] != 0xAA {
            return Err(BootError::Generic("Invalid boot sector signature"));
        }

        // Determine FAT type and get FAT size
        let fat_size = if bpb.fat_size_16 != 0 {
            bpb.fat_size_16 as u32
        } else {
            // FAT32
            let ext_bpb = unsafe { &*(boot_sector[36..].as_ptr() as *const Fat32ExtendedBPB) };
            ext_bpb.fat_size_32
        };

        let total_sectors = if bpb.total_sectors_16 != 0 {
            bpb.total_sectors_16 as u32
        } else {
            bpb.total_sectors_32
        };

        // Get root cluster (FAT32)
        let root_cluster = if bpb.fat_size_16 == 0 {
            let ext_bpb = unsafe { &*(boot_sector[36..].as_ptr() as *const Fat32ExtendedBPB) };
            ext_bpb.root_cluster
        } else {
            2 // FAT12/16 use cluster 2 for root
        };

        Ok(Self {
            bytes_per_sector: bpb.bytes_per_sector,
            sectors_per_cluster: bpb.sectors_per_cluster,
            reserved_sectors: bpb.reserved_sectors,
            num_fats: bpb.num_fats,
            fat_size,
            root_cluster,
            total_sectors,
        })
    }

    /// Read a file by path
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // TODO: Implement
        // 1. Parse path into components
        // 2. Navigate directory structure
        // 3. Find file entry
        // 4. Read clusters using FAT
        // 5. Return file data

        Err(BootError::Generic("FAT32 read_file not yet implemented"))
    }

    /// Get cluster chain from FAT
    fn get_cluster_chain(&self, start_cluster: u32) -> Result<Vec<u32>> {
        // TODO: Read FAT table and follow cluster chain
        let mut chain = Vec::new();
        chain.push(start_cluster);
        Ok(chain)
    }

    /// Calculate first sector of a cluster
    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        let root_dir_sectors = 0; // FAT32 has no fixed root directory
        let first_data_sector = self.reserved_sectors as u32
            + (self.num_fats as u32 * self.fat_size)
            + root_dir_sectors;
        first_data_sector + ((cluster - 2) * self.sectors_per_cluster as u32)
    }
}
