//! Backend de Armazenamento UEFI
//!
//! Implementa a trait `BlockDevice` do `src/fs` utilizando o protocolo
//! `EFI_BLOCK_IO_PROTOCOL`. Isso permite que o sistema de arquivos leia
//! setores de qualquer disco reconhecido pelo firmware.

use core::ffi::c_void;

use crate::{
    core::error::{BootError, FileSystemError, Result},
    fs::dev::BlockDevice,
    uefi::base::{Guid, Status},
};

/// GUID do Protocolo Block IO.
pub const BLOCK_IO_PROTOCOL_GUID: Guid = Guid::new(
    0x964e5b21,
    0x6459,
    0x11d2,
    [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b],
);

#[repr(C)]
struct BlockIoMedia {
    media_id:          u32,
    removable_media:   bool,
    media_present:     bool,
    logical_partition: bool,
    read_only:         bool,
    write_caching:     bool,
    block_size:        u32,
    io_align:          u32,
    last_block:        u64,
}

#[repr(C)]
struct BlockIoProtocol {
    revision:     u64,
    media:        *mut BlockIoMedia,
    reset:        extern "efiapi" fn(*mut BlockIoProtocol, bool) -> Status,
    read_blocks:  extern "efiapi" fn(*mut BlockIoProtocol, u32, u64, usize, *mut c_void) -> Status,
    write_blocks:
        extern "efiapi" fn(*mut BlockIoProtocol, u32, u64, usize, *const c_void) -> Status,
    flush_blocks: extern "efiapi" fn(*mut BlockIoProtocol) -> Status,
}

/// Wrapper seguro para um dispositivo de bloco UEFI.
pub struct UefiBlockDevice {
    protocol: *mut BlockIoProtocol,
    media:    *mut BlockIoMedia,
}

impl UefiBlockDevice {
    /// Tenta localizar o BlockIO associado ao Handle fornecido.
    pub fn new(handle: crate::uefi::Handle) -> Result<Self> {
        let bs = crate::uefi::system_table().boot_services();

        let protocol_ptr = bs
            .open_protocol(
                handle,
                &BLOCK_IO_PROTOCOL_GUID,
                crate::uefi::image_handle(),
                crate::uefi::Handle::null(),
                crate::uefi::table::boot::OPEN_PROTOCOL_GET_PROTOCOL,
            )
            .map_err(|_| BootError::FileSystem(FileSystemError::DeviceError))?;

        let protocol = protocol_ptr as *mut BlockIoProtocol;
        let media = unsafe { (*protocol).media };

        Ok(Self { protocol, media })
    }
}

impl BlockDevice for UefiBlockDevice {
    fn block_size(&self) -> u64 {
        unsafe { (*self.media).block_size as u64 }
    }

    fn num_blocks(&self) -> u64 {
        unsafe { (*self.media).last_block + 1 }
    }

    fn read_blocks(&mut self, lba: u64, buf: &mut [u8]) -> Result<()> {
        let media_id = unsafe { (*self.media).media_id };

        // Verifica alinhamento e tamanho
        let block_size = self.block_size() as usize;
        if buf.len() % block_size != 0 {
            return Err(BootError::FileSystem(FileSystemError::InvalidSize));
        }

        unsafe {
            ((*self.protocol).read_blocks)(
                self.protocol,
                media_id,
                lba,
                buf.len(),
                buf.as_mut_ptr() as *mut c_void,
            )
            .to_result()
            .map_err(|_| BootError::FileSystem(FileSystemError::ReadError))
        }
    }

    fn write_blocks(&mut self, lba: u64, buf: &[u8]) -> Result<()> {
        if unsafe { (*self.media).read_only } {
            return Err(BootError::FileSystem(FileSystemError::WriteError));
        }

        let media_id = unsafe { (*self.media).media_id };

        unsafe {
            ((*self.protocol).write_blocks)(
                self.protocol,
                media_id,
                lba,
                buf.len(),
                buf.as_ptr() as *const c_void,
            )
            .to_result()
            .map_err(|_| BootError::FileSystem(FileSystemError::WriteError))
        }
    }
}
