//! Carregador de InitFS (sistema de arquivos inicial)

use crate::error::Result;
use crate::fs::FileLoader;
use crate::types::LoadedFile;

/// Carregador de InitFS
pub struct InitFsLoader;

impl InitFsLoader {
    /// Carrega o InitFS se disponível
    ///
    /// # Argumentos
    /// * `file_loader` - Carregador de arquivos
    ///
    /// # Retorna
    /// Some(LoadedFile) se InitFS foi encontrado, None caso contrário
    pub fn load(file_loader: &mut FileLoader) -> Result<Option<LoadedFile>> {
        log::info!("Procurando InitFS...");

        match file_loader.try_load_file("initfs")? {
            Some(initfs) => {
                log::info!("InitFS encontrado: {} bytes em {:#x}", initfs.size, initfs.ptr);
                Ok(Some(initfs))
            }
            None => {
                log::warn!("InitFS não encontrado. Sistema iniciará sem userspace.");
                Ok(None)
            }
        }
    }
}
