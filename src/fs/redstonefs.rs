//! RedstoneFS - Sistema de Arquivos Personalizado
//!
//! Driver nativo para a partição do sistema operacional (/redstone-os).
//! Baseado em ZFS/Btrfs (COW, Checksums).

use alloc::boxed::Box;

use super::{
    dev::BlockDevice,
    vfs::{Directory, FileSystem},
};
use crate::core::error::{BootError, Result};

pub struct RedstoneFileSystem<D: BlockDevice> {
    #[allow(dead_code)]
    device: D,
}

impl<D: BlockDevice> RedstoneFileSystem<D> {
    pub fn mount(device: D) -> Result<Self> {
        // Verificar Magic Number no Superblock
        Ok(Self { device })
    }
}

impl<D: BlockDevice + 'static> FileSystem for RedstoneFileSystem<D> {
    fn root(&mut self) -> Result<Box<dyn Directory>> {
        Err(BootError::Generic("RedstoneFS ainda não implementado"))
    }

    fn name(&self) -> &str {
        "RFS"
    }
}
