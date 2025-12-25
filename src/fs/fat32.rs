//! Driver Nativo de Sistema de Arquivos FAT32
//!
//! Permite ler partições FAT32 diretamente, sem depender do UEFI.
//! Útil para montar partições extras que o firmware não reconheceu.

use alloc::{boxed::Box, vec};

use crate::{
    core::error::{BootError, Result},
    fs::{
        dev::BlockDevice,
        vfs::{Directory, FileSystem},
    },
};

/// BPB (Bios Parameter Block) para FAT32.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct BiosParameterBlock {
    jmp_boot:            [u8; 3],
    oem_name:            [u8; 8],
    bytes_per_sector:    u16,
    sectors_per_cluster: u8,
    reserved_sectors:    u16,
    num_fats:            u8,
    root_entry_count:    u16,
    total_sectors_16:    u16,
    media:               u8,
    fat_size_16:         u16,
    sectors_per_track:   u16,
    num_heads:           u16,
    hidden_sectors:      u32,
    total_sectors_32:    u32,
}

/// Extended BPB.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Fat32Ext {
    fat_size_32:        u32,
    ext_flags:          u16,
    fs_version:         u16,
    root_cluster:       u32,
    fs_info:            u16,
    backup_boot_sector: u16,
    reserved:           [u8; 12],
    drive_number:       u8,
    reserved1:          u8,
    boot_signature:     u8,
    volume_id:          u32,
    volume_label:       [u8; 11],
    fs_type:            [u8; 8],
}

pub struct Fat32FileSystem<D: BlockDevice> {
    device:              D,
    // Metadados do FS em cache
    fat_start_lba:       u64,
    data_start_lba:      u64,
    sectors_per_cluster: u64,
    root_cluster:        u32,
}

impl<D: BlockDevice> Fat32FileSystem<D> {
    /// Tenta montar um volume FAT32 a partir de um dispositivo de bloco.
    pub fn mount(mut device: D) -> Result<Self> {
        let mut buf = vec![0u8; 512];
        device
            .read_blocks(0, &mut buf)
            .map_err(|_| BootError::FileSystem(crate::core::error::FileSystemError::ReadError))?;

        // Validação de assinatura
        if buf[510] != 0x55 || buf[511] != 0xAA {
            return Err(BootError::FileSystem(
                crate::core::error::FileSystemError::InvalidSignature,
            ));
        }

        // Parse manual simplificado ou via struct (unsafe cast)
        let bpb = unsafe { &*(buf.as_ptr() as *const BiosParameterBlock) };

        // Verifica se é FAT32
        if bpb.fat_size_16 != 0 {
            return Err(BootError::FileSystem(
                crate::core::error::FileSystemError::UnsupportedFsType,
            ));
        }

        // TODO: Completar cálculos de LBA

        Ok(Self {
            device,
            fat_start_lba: 0,  // Placeholder
            data_start_lba: 0, // Placeholder
            sectors_per_cluster: bpb.sectors_per_cluster as u64,
            root_cluster: 2, // Geralmente 2, mas deve ser lido do ext_bpb
        })
    }
}

// Implementação VFS (Stubs para compilação)
impl<D: BlockDevice + 'static> FileSystem for Fat32FileSystem<D> {
    fn root(&mut self) -> Result<Box<dyn Directory>> {
        Err(BootError::Generic("FAT32 nativo ainda não implementado"))
    }

    fn name(&self) -> &str {
        "FAT32_NATIVE"
    }
}
