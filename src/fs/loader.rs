//! Carregador de Arquivos de Alto Nível
//!
//! Utilitário para localizar e ler arquivos completos para a memória.
//! Abstrai a abertura de diretórios e leitura em chunks.

use alloc::{boxed::Box, vec::Vec};

use super::FileSystem;
use crate::core::{
    error::{BootError, FileSystemError, Result},
    types::LoadedFile,
};

/// Abstração para carregamento de arquivos.
pub struct FileLoader<'a> {
    fs: &'a mut dyn FileSystem,
}

impl<'a> FileLoader<'a> {
    /// Cria um novo loader vinculado a um sistema de arquivos.
    pub fn new(fs: &'a mut dyn FileSystem) -> Self {
        Self { fs }
    }

    /// Verifica se um arquivo existe sem carregá-lo.
    pub fn file_exists(&mut self, path: &str) -> bool {
        // Tenta abrir a raiz e depois o arquivo
        if let Ok(mut root) = self.fs.root() {
            return root.open_file(path).is_ok();
        }
        false
    }

    /// Carrega um arquivo inteiro para a memória.
    ///
    /// # Retorna
    /// `LoadedFile` contendo o ponteiro físico e o tamanho.
    ///
    /// # Memory Leak Intencional
    /// O buffer alocado é "vazado" (`Box::leak`) para garantir que a memória
    /// permaneça válida quando passarmos o ponteiro para o Kernel, sobrevivendo
    /// ao fim da execução desta função.
    pub fn load_file(&mut self, path: &str) -> Result<LoadedFile> {
        let mut root = self.fs.root()?;
        let mut file = root
            .open_file(path)
            .map_err(|_| BootError::FileSystem(FileSystemError::FileNotFound))?;

        // Aloca buffer com tamanho do arquivo
        let size = file.metadata()?.size as usize;
        let mut buffer = Vec::with_capacity(size);
        buffer.resize(size, 0);

        // Lê conteúdo
        let read_len = file.read(&mut buffer)?;
        if read_len != size {
            // Opcional: Warning se leu menos que o tamanho reportado
        }

        // Transforma o Vec em Box<[u8]> e "vaza" para obter referência 'static
        let leaked_ref = Box::leak(buffer.into_boxed_slice());
        let ptr = leaked_ref.as_ptr() as u64;

        crate::println!("Arquivo carregado: {} ({} bytes) @ {:#x}", path, size, ptr);

        Ok(LoadedFile { ptr, size })
    }
}
