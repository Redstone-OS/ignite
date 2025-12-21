//! Backend VFS para UEFI Simple File System
//!
//! Envolve o protocolo nativo da UEFI para que possa ser usado através da trait
//! `Vfs`. É o driver principal usado para carregar o kernel da partição ESP.

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use super::path::normalize_path;
use crate::{
    core::error::{BootError, FileSystemError, Result},
    fs::vfs::{Directory, File, FileSystem, Metadata},
    uefi::{
        Char16,
        base::{Guid, Status},
        proto::media::{
            file::{
                FILE_INFO_GUID, FILE_MODE_CREATE, FILE_MODE_READ, FILE_MODE_WRITE, FileProtocol,
            },
            fs::SimpleFileSystemProtocol,
        },
    },
};

// --- Estruturas Wrapper ---

pub struct UefiFileSystem<'a> {
    protocol: &'a mut SimpleFileSystemProtocol,
}

impl<'a> UefiFileSystem<'a> {
    pub fn new(protocol: &'a mut SimpleFileSystemProtocol) -> Self {
        Self { protocol }
    }
}

impl<'a> FileSystem for UefiFileSystem<'a> {
    fn root(&mut self) -> Result<Box<dyn Directory>> {
        let root_ptr = self.protocol.open_volume()?;
        Ok(Box::new(UefiDir { protocol: root_ptr }))
    }

    fn name(&self) -> &str {
        "UEFI_SIMPLE_FS"
    }
}

pub struct UefiFile {
    protocol: *mut FileProtocol,
}

impl File for UefiFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut size = buf.len();
        unsafe {
            ((*self.protocol).read)(self.protocol, &mut size, buf.as_mut_ptr() as *mut _)
                .to_result()
                .map_err(|_| BootError::FileSystem(FileSystemError::ReadError))?;
        }
        Ok(size)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut size = buf.len();
        unsafe {
            ((*self.protocol).write)(self.protocol, &mut size, buf.as_ptr() as *const _)
                .to_result()
                .map_err(|_| BootError::FileSystem(FileSystemError::WriteError))?;
        }
        Ok(size)
    }

    fn seek(&mut self, offset: u64) -> Result<u64> {
        unsafe {
            ((*self.protocol).set_position)(self.protocol, offset)
                .to_result()
                .map_err(|_| BootError::FileSystem(FileSystemError::SeekError))?;

            let mut pos = 0u64;
            ((*self.protocol).get_position)(self.protocol, &mut pos)
                .to_result()
                .map(|_| pos)
                .map_err(|_| BootError::FileSystem(FileSystemError::SeekError))
        }
    }

    fn metadata(&self) -> Result<Metadata> {
        // Obter tamanho (Simplificado: assumindo Info genérico)
        // Em produção real, alocaríamos o buffer para EFI_FILE_INFO e faríamos o parse.
        // Como não temos alloc::vec no core uefi::proto, deixamos stub seguro.
        Ok(Metadata {
            size:        0,
            is_dir:      false,
            is_readonly: false,
        })
    }

    fn close(&mut self) -> Result<()> {
        unsafe {
            ((*self.protocol).close)(self.protocol)
                .to_result()
                .map_err(|_| BootError::FileSystem(FileSystemError::VolumeOpenError))
        }
    }
}

// Implementação de Drop para garantir fechamento
impl Drop for UefiFile {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub struct UefiDir {
    protocol: *mut FileProtocol, // Raw pointer pois FileProtocol não é Send/Sync
}

impl Directory for UefiDir {
    fn open_file(&mut self, path: &str) -> Result<Box<dyn File>> {
        let path_norm = normalize_path(path);
        let mut file_ptr = core::ptr::null_mut();

        // Conversão de String para UCS-2 (Char16)
        // Isso é caro e requer alocação, mas necessário para UEFI.
        let path_utf16: Vec<u16> = path_norm
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect();

        unsafe {
            ((*self.protocol).open)(
                self.protocol,
                &mut file_ptr,
                path_utf16.as_ptr(),
                FILE_MODE_READ,
                0,
            )
            .to_result()
            .map_err(|_| BootError::FileSystem(FileSystemError::FileNotFound))?;
        }

        Ok(Box::new(UefiFile { protocol: file_ptr }))
    }

    fn open_dir(&mut self, path: &str) -> Result<Box<dyn Directory>> {
        // Em UEFI, abrir diretório é igual a abrir arquivo
        let path_norm = normalize_path(path);
        let mut dir_ptr = core::ptr::null_mut();
        let path_utf16: Vec<u16> = path_norm
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect();

        unsafe {
            ((*self.protocol).open)(
                self.protocol,
                &mut dir_ptr,
                path_utf16.as_ptr(),
                FILE_MODE_READ,
                0, // Atributos ignorados na abertura de leitura
            )
            .to_result()
            .map_err(|_| BootError::FileSystem(FileSystemError::FileNotFound))?;
        }

        Ok(Box::new(UefiDir { protocol: dir_ptr }))
    }

    fn list(&mut self) -> Result<Vec<String>> {
        // Implementação futura: ler entradas do diretório repetidamente
        Ok(Vec::new())
    }
}

impl Drop for UefiDir {
    fn drop(&mut self) {
        unsafe {
            ((*self.protocol).close)(self.protocol);
        }
    }
}
