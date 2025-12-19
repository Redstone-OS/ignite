//! Carregador de arquivos do sistema de arquivos UEFI

extern crate alloc;

use uefi::{
    prelude::*,
    proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode},
};

use crate::{
    error::{BootError, FileSystemError, Result},
    memory::MemoryAllocator,
    types::LoadedFile,
};

/// Carregador de arquivos UEFI
pub struct FileLoader<'a> {
    root:      Directory,
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
    /// * `filename` - Nome do arquivo a carregar (aceita qualquer &str)
    ///
    /// # Retorna
    /// Informações sobre o arquivo carregado (ponteiro e tamanho)
    pub fn load_file(&mut self, filename: &str) -> Result<LoadedFile> {
        log::info!("Carregando arquivo: {}", filename);

        // Sanitizar caminho do arquivo
        let mut path_str = alloc::string::String::from(filename);

        // 1. Remover prefixo "boot():" se existir
        if path_str.starts_with("boot():") {
            path_str = alloc::string::String::from(&path_str[7..]);
        }

        // 2. Substituir / por \ (padrão UEFI)
        path_str = path_str.replace('/', "\\");

        // 3. Remover \ inicial se houver (muitos firmwares preferem caminhos relativos à raiz)
        if path_str.starts_with('\\') {
            path_str.remove(0);
        }

        log::info!("Caminho processado: '{}' -> '{}'", filename, path_str);

        // Converter filename UTF-8 para UTF-16 (CStr16)
        // UEFI usa UTF-16, então precisamos converter
        use uefi::CStr16;

        // Criar buffer UTF-16 no stack (256 u16s = 512 bytes)
        let mut utf16_buf = [0u16; 256];
        let filename_cstr = CStr16::from_str_with_buf(path_str.as_str(), &mut utf16_buf)
            .map_err(|_| BootError::FileSystem(FileSystemError::InvalidPath))?;

        // Abrir arquivo
        let mut file = self
            .root
            .open(filename_cstr, FileMode::Read, FileAttribute::empty())
            .map_err(|_| {
                log::error!("Arquivo não encontrado: {}", filename);
                BootError::FileSystem(FileSystemError::FileNotFound)
            })?
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
            ptr:  file_ptr,
            size: file_size,
        })
    }

    /// Tenta carregar um arquivo, retornando None se não encontrado
    ///
    /// # Argumentos
    /// * `filename` - Nome do arquivo a carregar
    pub fn try_load_file(&mut self, filename: &str) -> Result<Option<LoadedFile>> {
        match self.load_file(filename) {
            Ok(file) => Ok(Some(file)),
            Err(BootError::FileSystem(FileSystemError::FileNotFound)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
