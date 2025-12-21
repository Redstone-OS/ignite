//! Virtual File System (VFS)
//!
//! Define as interfaces abstratas para sistemas de arquivos.
//! O Bootloader opera sobre estas traits, ignorando se o arquivo vem
//! do UEFI, de um driver FAT32 nativo ou via rede.

use alloc::{boxed::Box, string::String, vec::Vec};

use crate::core::error::Result;

/// Metadados básicos de arquivo.
#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub size:        u64,
    pub is_dir:      bool,
    pub is_readonly: bool,
}

/// Representa um arquivo aberto.
pub trait File {
    /// Lê bytes do arquivo para o buffer.
    /// Retorna o número de bytes lidos.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Escreve bytes do buffer no arquivo.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Move o cursor de leitura/escrita.
    fn seek(&mut self, offset: u64) -> Result<u64>;

    /// Obtém metadados.
    fn metadata(&self) -> Result<Metadata>;

    /// Fecha o arquivo (opcional, pois Drop deve lidar com isso).
    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Representa um diretório aberto.
pub trait Directory {
    /// Abre um arquivo dentro deste diretório.
    fn open_file(&mut self, path: &str) -> Result<Box<dyn File>>;

    /// Abre um subdiretório.
    fn open_dir(&mut self, path: &str) -> Result<Box<dyn Directory>>;

    /// Lista entradas do diretório (Simplificado para Vec de Strings por
    /// enquanto).
    fn list(&mut self) -> Result<Vec<String>>;
}

/// Representa um Sistema de Arquivos montado.
pub trait FileSystem {
    /// Abre o diretório raiz.
    fn root(&mut self) -> Result<Box<dyn Directory>>;

    /// Nome do driver (ex: "FAT32", "UEFI_SIMPLE_FS").
    fn name(&self) -> &str;
}
