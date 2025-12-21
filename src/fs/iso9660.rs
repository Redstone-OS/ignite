//! Driver de sistema de arquivos ISO9660
//!
//! Lê sistemas de arquivos ISO9660 CD/DVD

use alloc::vec::Vec;

use crate::core::error::{BootError, Result};

/// Descritor de volume ISO9660
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct VolumeDescriptor {
    type_code:  u8,
    identifier: [u8; 5], // "CD001"
    version:    u8,
}

/// Descritor de volume primário
#[repr(C, packed)]
struct PrimaryVolumeDescriptor {
    type_code:              u8,
    identifier:             [u8; 5],
    version:                u8,
    unused1:                u8,
    system_id:              [u8; 32],
    volume_id:              [u8; 32],
    unused2:                [u8; 8],
    volume_space_size:      [u8; 8], // Ambos-endian
    unused3:                [u8; 32],
    volume_set_size:        [u8; 4],
    volume_sequence_number: [u8; 4],
    logical_block_size:     [u8; 4],
    path_table_size:        [u8; 8],
    // ... more fields
}

/// Registro de diretório
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct DirectoryRecord {
    length:                 u8,
    extended_attr_length:   u8,
    extent_location:        [u8; 8], // Ambos-endian
    data_length:            [u8; 8], // Ambos-endian
    recording_date_time:    [u8; 7],
    file_flags:             u8,
    file_unit_size:         u8,
    interleave_gap_size:    u8,
    volume_sequence_number: [u8; 4],
    name_length:            u8,
    // Seguido pelo nome
}

/// Sistema de arquivos ISO9660
pub struct Iso9660FileSystem {
    block_size:      u32,
    root_dir_extent: u32,
    root_dir_size:   u32,
}

impl Iso9660FileSystem {
    /// Montar ISO9660 a partir de descritores de volume
    pub fn mount(descriptors: &[u8]) -> Result<Self> {
        // Descritores de volume ISO9660 começam no setor 16 (0x8000 bytes)
        const SECTOR_SIZE: usize = 2048;
        let mut offset = 16 * SECTOR_SIZE;

        while offset + SECTOR_SIZE <= descriptors.len() {
            let vd = unsafe { &*(descriptors[offset..].as_ptr() as *const VolumeDescriptor) };

            // Verificar identificador "CD001"
            if &vd.identifier != b"CD001" {
                return Err(BootError::Generic("Invalid ISO9660 identifier"));
            }

            if vd.type_code == 1 {
                // Descritor de volume primário
                let pvd =
                    unsafe { &*(descriptors[offset..].as_ptr() as *const PrimaryVolumeDescriptor) };

                // Ler tamanho do bloco (little-endian do campo both-endian)
                let block_size = u32::from_le_bytes([
                    pvd.logical_block_size[0],
                    pvd.logical_block_size[1],
                    pvd.logical_block_size[2],
                    pvd.logical_block_size[3],
                ]);

                // Diretório raiz está no offset 156 no PVD
                // TODO: Parsear registro de diretório raiz corretamente

                return Ok(Self {
                    block_size,
                    root_dir_extent: 0,
                    root_dir_size: 0,
                });
            } else if vd.type_code == 255 {
                // Terminador do conjunto de descritores de volume
                break;
            }

            offset += SECTOR_SIZE;
        }

        Err(BootError::Generic(
            "Descritor de Volume Primário não encontrado",
        ))
    }

    /// Ler um arquivo pelo caminho
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // TODO: Implementar
        // 1. Começar do diretório raiz
        // 2. Parsear componentes do caminho
        // 3. Navegar árvore de diretório
        // 4. Encontrar arquivo
        // 5. Ler extents

        Err(BootError::Generic(
            "ISO9660 read_file ainda não implementado",
        ))
    }
}
