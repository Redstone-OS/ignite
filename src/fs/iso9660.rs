//! ISO9660 Filesystem Driver
//!
//! Reads ISO9660 CD/DVD filesystems

use alloc::vec::Vec;

use crate::error::{BootError, Result};

/// ISO9660 Volume Descriptor
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct VolumeDescriptor {
    type_code:  u8,
    identifier: [u8; 5], // "CD001"
    version:    u8,
}

/// Primary Volume Descriptor
#[repr(C, packed)]
struct PrimaryVolumeDescriptor {
    type_code:              u8,
    identifier:             [u8; 5],
    version:                u8,
    unused1:                u8,
    system_id:              [u8; 32],
    volume_id:              [u8; 32],
    unused2:                [u8; 8],
    volume_space_size:      [u8; 8], // Both-endian
    unused3:                [u8; 32],
    volume_set_size:        [u8; 4],
    volume_sequence_number: [u8; 4],
    logical_block_size:     [u8; 4],
    path_table_size:        [u8; 8],
    // ... more fields
}

/// Directory Record
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DirectoryRecord {
    length:                 u8,
    extended_attr_length:   u8,
    extent_location:        [u8; 8], // Both-endian
    data_length:            [u8; 8], // Both-endian
    recording_date_time:    [u8; 7],
    file_flags:             u8,
    file_unit_size:         u8,
    interleave_gap_size:    u8,
    volume_sequence_number: [u8; 4],
    name_length:            u8,
    // Followed by name
}

/// ISO9660 filesystem
pub struct Iso9660FileSystem {
    block_size:      u32,
    root_dir_extent: u32,
    root_dir_size:   u32,
}

impl Iso9660FileSystem {
    /// Mount ISO9660 from volume descriptors
    pub fn mount(descriptors: &[u8]) -> Result<Self> {
        // ISO9660 volume descriptors start at sector 16 (0x8000 bytes)
        const SECTOR_SIZE: usize = 2048;
        let mut offset = 16 * SECTOR_SIZE;

        while offset + SECTOR_SIZE <= descriptors.len() {
            let vd = unsafe { &*(descriptors[offset..].as_ptr() as *const VolumeDescriptor) };

            // Check identifier "CD001"
            if &vd.identifier != b"CD001" {
                return Err(BootError::Generic("Invalid ISO9660 identifier"));
            }

            if vd.type_code == 1 {
                // Primary Volume Descriptor
                let pvd =
                    unsafe { &*(descriptors[offset..].as_ptr() as *const PrimaryVolumeDescriptor) };

                // Read block size (little-endian from both-endian field)
                let block_size = u32::from_le_bytes([
                    pvd.logical_block_size[0],
                    pvd.logical_block_size[1],
                    pvd.logical_block_size[2],
                    pvd.logical_block_size[3],
                ]);

                // Root directory is at offset 156 in PVD
                // TODO: Parse root directory record properly

                return Ok(Self {
                    block_size,
                    root_dir_extent: 0,
                    root_dir_size: 0,
                });
            } else if vd.type_code == 255 {
                // Volume Descriptor Set Terminator
                break;
            }

            offset += SECTOR_SIZE;
        }

        Err(BootError::Generic("Primary Volume Descriptor not found"))
    }

    /// Read a file by path
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // TODO: Implement
        // 1. Start from root directory
        // 2. Parse path components
        // 3. Navigate directory tree
        // 4. Find file
        // 5. Read extents

        Err(BootError::Generic("ISO9660 read_file not yet implemented"))
    }
}
