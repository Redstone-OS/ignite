//! Abstração de Dispositivos de Bloco
//!
//! Interface necessária para drivers nativos (FAT32, RedstoneFS) lerem discos
//! cru.

use crate::core::error::Result;

/// Trait para dispositivos que permitem leitura/escrita em blocos (setores).
pub trait BlockDevice {
    /// Lê blocos do dispositivo.
    fn read_blocks(&mut self, lba: u64, buf: &mut [u8]) -> Result<()>;

    /// Escreve blocos no dispositivo.
    fn write_blocks(&mut self, lba: u64, buf: &[u8]) -> Result<()>;

    /// Tamanho do bloco em bytes (geralmente 512 ou 4096).
    fn block_size(&self) -> u64;

    /// Número total de blocos.
    fn num_blocks(&self) -> u64;
}
