//! Carregador de arquivos do sistema de arquivos UEFI

use crate::error::{BootError, FileSystemError, Result};
use crate::memory::MemoryAllocator;
use crate::types::LoadedFile;
use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode};

/// Carregador de arquivos UEFI
pub struct FileLoader<'a> {
    root: Directory,
    allocator: &'a MemoryAllocator<'a>,
}

impl<'a> FileLoader<'a> {
    /// Cria um novo carregador de arquivos
    ///
    /// # Argumentos
    /// * `boot_services` - Serviços de boot UEFI
    /// * `image_handle` - Handle da imagem do bootloader
    /// * `allocator` - Alocador de memória
    pub fn new(
        boot_services: &BootServices,
        image_handle: Handle,
        allocator: &'a MemoryAllocator<'a>,
    ) -> Result<Self> {
        // Obter sistema de arquivos da imagem
        let root = boot_services
            .get_image_file_system(image_handle)
            .map_err(|_| BootError::FileSystem(FileSystemError::VolumeOpenError))?
            .open_volume()
            .map_err(|_| BootError::FileSystem(FileSystemError::VolumeOpenError))?;

        Ok(Self { root, allocator })
    }

    /// Carrega um arquivo na memória
    ///
    /// # Argumentos
    /// * `filename` - Nome do arquivo a carregar
    ///
    /// # Retorna
    /// Informações sobre o arquivo carregado (ponteiro e tamanho)
    pub fn load_file(&mut self, filename: &'static str) -> Result<LoadedFile> {
        log::info!("Carregando arquivo: {}", filename);

        // Converter filename para CStr16
        let filename_cstr = match filename {
            "forge" => cstr16!("forge"),
            "initfs" => cstr16!("initfs"),
            _ => return Err(BootError::FileSystem(FileSystemError::InvalidPath)),
        };

        // Abrir arquivo
        let mut file = self
            .root
            .open(filename_cstr, FileMode::Read, FileAttribute::empty())
            .map_err(|_| BootError::FileSystem(FileSystemError::FileNotFound(filename)))?
            .into_regular_file()
            .ok_or(BootError::FileSystem(FileSystemError::NotRegularFile))?;

        // Obter tamanho do arquivo
        let mut info_buf = [0u8; 128];
        let file_info = file
            .get_info::<FileInfo>(&mut info_buf)
            .map_err(|_| BootError::FileSystem(FileSystemError::ReadError))?;
        let file_size = file_info.file_size() as usize;

        log::info!("Arquivo encontrado: {} bytes", file_size);

        // Alocar memória para o arquivo
        let file_pages = MemoryAllocator::pages_for_size(file_size);
        let file_ptr = self.allocator.allocate_any(file_pages)?;

        // Ler arquivo para memória
        let file_slice = unsafe { core::slice::from_raw_parts_mut(file_ptr as *mut u8, file_size) };
        file.read(file_slice)
            .map_err(|_| BootError::FileSystem(FileSystemError::ReadError))?;

        log::info!("Arquivo carregado em {:#x}", file_ptr);

        Ok(LoadedFile {
            ptr: file_ptr,
            size: file_size,
        })
    }

    /// Tenta carregar um arquivo, retornando None se não encontrado
    ///
    /// # Argumentos
    /// * `filename` - Nome do arquivo a carregar
    pub fn try_load_file(&mut self, filename: &'static str) -> Result<Option<LoadedFile>> {
        match self.load_file(filename) {
            Ok(file) => Ok(Some(file)),
            Err(BootError::FileSystem(FileSystemError::FileNotFound(_))) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
