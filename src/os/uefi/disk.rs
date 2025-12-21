use alloc::vec::Vec;
use core::slice;

use syscall::{EINVAL, EIO, Error, Result};
use uefi::{
    Guid,
    proto::{Protocol, media::block::BlockIO as UefiBlockIo},
};

use crate::redstonefs::{BLOCK_SIZE, Disk, RECORD_SIZE};

pub enum DiskOrFileEfi {
    Disk(DiskEfi),
    File(Vec<u8>),
}

impl crate::redstonefs::Disk for DiskOrFileEfi {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> syscall::Result<usize> {
        unsafe {
            match self {
                DiskOrFileEfi::Disk(disk_efi) => disk_efi.read_at(block, buffer),
                DiskOrFileEfi::File(data) => {
                    buffer.copy_from_slice(
                        &data[(block * crate::redstonefs::BLOCK_SIZE) as usize
                            ..(block * crate::redstonefs::BLOCK_SIZE) as usize + buffer.len()],
                    );
                    Ok(buffer.len())
                },
            }
        }
    }

    fn write_at(&mut self, _block: u64, _buffer: &[u8]) -> syscall::Result<usize> {
        unreachable!()
    }

    fn size(&mut self) -> syscall::Result<u64> {
        unreachable!()
    }
}

pub struct DiskEfi(pub &'static mut UefiBlockIo, &'static mut [u8]);

use uefi::Identify;
use uefi_services::println;

unsafe impl Identify for DiskEfi {
    const GUID: Guid = UefiBlockIo::GUID;
}

impl Protocol for DiskEfi {
    // new() was used to construct? Protocol trait doesn't have new.
    // The previous code had `fn new(...)` in impl Protocol.
    // This was likely a helper in uefi_std or custom.
    // I should move new() to inherent impl.
}

impl DiskEfi {
    fn new(inner: &'static mut UefiBlockIo) -> Self {
        // Hack to get aligned buffer
        let block = unsafe {
            let ptr = super::alloc_zeroed_page_aligned(RECORD_SIZE as usize);
            slice::from_raw_parts_mut(ptr, RECORD_SIZE as usize)
        };

        Self(inner, block)
    }
}

impl Disk for DiskEfi {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize> {
        unsafe {
            // Optimization for live disks
            /* if let Some(live) = crate::LIVE_OPT {
                if block >= live.0 {
                    let start = ((block - live.0) * BLOCK_SIZE) as usize;
                    let end = start + buffer.len();
                    if end <= live.1.len() {
                        buffer.copy_from_slice(&live.1[start..end]);
                        return Ok(buffer.len());
                    }
                }
            } */

            let media = self.0.media();
            // Use aligned buffer if necessary
            let mut ptr = buffer.as_mut_ptr();
            // This is a check for alignment.
            // If uefi-rs read_blocks handles slice, we can use slice.
            // But we need to handle alignment MANUALLY if the device requires it.
            // "io_align" 0 or 1 means no alignment.
            
            if media.io_align() > 1 {
                if (ptr as usize) % (media.io_align() as usize) != 0 {
                    if buffer.len() <= self.1.len() {
                        ptr = self.1.as_mut_ptr();
                    } else {
                        println!(
                            "DiskEfi::read_at 0x{:X} requires alignment, ptr = 0x{:p}, len = 0x{:x}",
                            block,
                            ptr,
                            buffer.len()
                        );
                        return Err(Error::new(EINVAL));
                    }
                }
            }

            let block_size = media.block_size() as u64;
            let lba = block * BLOCK_SIZE / block_size;

            // Using read_blocks safe wrapper
            // It expects &mut [u8].
            // We create a slice from the possibly aligned ptr.
            let len = buffer.len();
            let slice = slice::from_raw_parts_mut(ptr, len);
            
            match self.0.read_blocks(media.media_id(), lba, slice) {
                Ok(_) => {
                    // Copy to original buffer if using aligned buffer
                    if ptr != buffer.as_mut_ptr() {
                        let (left, _) = self.1.split_at(buffer.len());
                        buffer.copy_from_slice(left);
                    }
                    Ok(len)
                },
                Err(e) => {
                    uefi_services::println!("DiskEfi::read_at 0x{:X} failed: {:?}", block, e);
                    Err(Error::new(EIO))
                }
            }
        }
    }

    fn write_at(&mut self, block: u64, _buffer: &[u8]) -> Result<usize> {
        println!("DiskEfi::write_at 0x{:X} not implemented", block);
        Err(Error::new(EIO))
    }

    fn size(&mut self) -> Result<u64> {
        println!("DiskEfi::size not implemented");
        Err(Error::new(EIO))
    }
}
