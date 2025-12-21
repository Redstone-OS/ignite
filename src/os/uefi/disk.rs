use alloc::vec::Vec;
use core::slice;

use syscall::{EINVAL, EIO, Error, Result};
use crate::uefi::{

    Guid,
    proto::{Protocol, media::block::BlockIO as UefiBlockIo},
};

use crate::fs::redstonefs::{BLOCK_SIZE, Disk, RECORD_SIZE};

pub enum DiskOrFileEfi {
    Disk(DiskEfi),
    File(Vec<u8>),
}

impl crate::fs::redstonefs::Disk for DiskOrFileEfi {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> syscall::Result<usize> {
        unsafe {
            match self {
                DiskOrFileEfi::Disk(disk_efi) => disk_efi.read_at(block, buffer),
                DiskOrFileEfi::File(data) => {
                    buffer.copy_from_slice(
                        &data[(block * crate::fs::redstonefs::BLOCK_SIZE) as usize
                            ..(block * crate::fs::redstonefs::BLOCK_SIZE) as usize + buffer.len()],
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

pub struct DiskEfi(pub &'static mut UefiBlockIo, pub &'static mut [u8]);

use uefi::Identify;

unsafe impl Identify for DiskEfi {
    const GUID: Guid = UefiBlockIo::GUID;
}

impl Protocol for DiskEfi {
    // new() foi usado para construir? Protocol trait não tem new.
    // O código anterior tinha `fn new(...)` em impl Protocol.
    // Isso provavelmente era um auxiliar em uefi_std ou customizado.
    // Eu devo mover new() para impl inerente.
}

impl DiskEfi {
    fn new(inner: &'static mut UefiBlockIo) -> Self {
        // Hack para obter buffer alinhado
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
            // Otimização para discos live
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
            // Usar buffer alinhado se necessário
            let mut ptr = buffer.as_mut_ptr();
            // Esta é uma checagem para alinhamento.
            // Se uefi-rs read_blocks lida com slice, podemos usar slice.
            // Mas precisamos lidar com alinhamento MANUALMENTE se o dispositivo requerer.
            // "io_align" 0 ou 1 significa sem alinhamento.
            
            if media.io_align() > 1 {
                if (ptr as usize) % (media.io_align() as usize) != 0 {
                    if buffer.len() <= self.1.len() {
                        ptr = self.1.as_mut_ptr();
                    } else {
                        // println!("DiskEfi::read_at 0x{:X} requires alignment", block);
                        return Err(Error::new(EINVAL));
                    }
                }
            }

            let block_size = media.block_size() as u64;
            let lba = block * BLOCK_SIZE / block_size;

            // Usando wrapper seguro read_blocks
            // Ele espera &mut [u8].
            // Criamos um slice do ponteiro possivelmente alinhado.
            let len = buffer.len();
            let slice = slice::from_raw_parts_mut(ptr, len);
            
            match self.0.read_blocks(media.media_id(), lba, slice) {
                Ok(_) => {
                    // Copiar para buffer original se usando buffer alinhado
                    if ptr != buffer.as_mut_ptr() {
                        let (left, _) = self.1.split_at(buffer.len());
                        buffer.copy_from_slice(left);
                    }
                    Ok(len)
                },
                Err(e) => {
                    // uefi_services::println!("DiskEfi::read_at 0x{:X} failed: {:?}", block, e);
                    Err(Error::new(EIO))
                }
            }
        }
    }

    fn write_at(&mut self, block: u64, _buffer: &[u8]) -> Result<usize> {
        // println!("DiskEfi::write_at 0x{:X} not implemented", block);
        Err(Error::new(EIO))
    }

    fn size(&mut self) -> Result<u64> {
        // println!("DiskEfi::size not implemented");
        Err(Error::new(EIO))
    }
}
