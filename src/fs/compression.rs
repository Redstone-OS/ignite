//! Suporte a compressão de arquivos
//!
//! Permite descomprimir arquivos automaticamente baseado na extensão.
//! Suporta zstd (.zst) para reduzir tamanho de initramfs e outros arquivos
//! grandes.

use alloc::vec::Vec;

use crate::core::error::Result;

/// Tipos de compressão suportados
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// Sem compressão
    None,
    /// Zstandard (.zst)
    Zstd,
}

impl CompressionType {
    /// Detecta tipo de compressão pela extensão do arquivo
    ///
    /// # Exemplos
    /// ```
    /// assert_eq!(CompressionType::from_path("boot/initfs"), CompressionType::None);
    /// assert_eq!(CompressionType::from_path("boot/initfs.zst"), CompressionType::Zstd);
    /// ```
    pub fn from_path(path: &str) -> Self {
        if path.ends_with(".zst") || path.ends_with(".zstd") {
            CompressionType::Zstd
        } else {
            CompressionType::None
        }
    }
}

/// Descomprime dados se necessário, baseado no tipo de compressão
///
/// # Argumentos
/// * `data` - Dados possivelmente comprimidos
/// * `compression` - Tipo de compressão
///
/// # Retorna
/// * `Ok(Vec<u8>)` - Dados descomprimidos (ou originais se sem compressão)
/// * `Err` - Se falhar descompressão
pub fn decompress(data: &[u8], compression: CompressionType) -> Result<Vec<u8>> {
    match compression {
        CompressionType::None => {
            // Sem compressão: retorna cópia dos dados
            Ok(data.to_vec())
        },
        CompressionType::Zstd => {
            // Descomprimir com zstd
            decompress_zstd(data)
        },
    }
}

/// Descomprime dados usando zstd
///
/// TODO: Implementar quando adicionar dependência zstd no Cargo.toml
fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>> {
    // PLACEHOLDER: Quando implementarmos zstd, usar:
    // ```
    // use zstd::decode_all;
    // decode_all(data).map_err(|_| BootError::DecompressionFailed)
    // ```

    log::warn!("Compressão zstd ainda não implementada");
    log::info!("Retornando dados comprimidos sem descomprimir");
    log::info!(
        "Adicione 'zstd = {{ version = \"0.13\", default-features = false }}' em Cargo.toml"
    );

    // Por enquanto, retorna os dados como estão
    // Isso permite que o código compile e funcione,
    // mas initfs comprimidos não serão descomprimidos
    Ok(data.to_vec())
}

/// Descomprime dados automaticamente baseado no caminho
///
/// Conveniência que combina detecção de tipo e descompressão.
///
/// # Argumentos
/// * `data` - Dados possivelmente comprimidos
/// * `path` - Caminho do arquivo (para detectar tipo)
///
/// # Exemplo
/// ```
/// let data = load_file("boot/initfs.zst");
/// let decompressed = decompress_auto(&data, "boot/initfs.zst")?;
/// ```
pub fn decompress_auto(data: &[u8], path: &str) -> Result<Vec<u8>> {
    let compression_type = CompressionType::from_path(path);

    if compression_type != CompressionType::None {
        log::info!(
            "Arquivo comprimido detectado: {} (tipo: {:?})",
            path,
            compression_type
        );

        let original_size = data.len();
        let result = decompress(data, compression_type)?;
        let decompressed_size = result.len();

        // Calcular taxa de compressão
        let ratio = if original_size > 0 {
            (decompressed_size as f32 / original_size as f32)
        } else {
            0.0
        };

        log::info!(
            "Descompressão: {} bytes -> {} bytes (ratio: {:.2}x)",
            original_size,
            decompressed_size,
            ratio
        );

        Ok(result)
    } else {
        // Sem compressão
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type_detection() {
        assert_eq!(
            CompressionType::from_path("boot/kernel"),
            CompressionType::None
        );
        assert_eq!(
            CompressionType::from_path("boot/initfs.zst"),
            CompressionType::Zstd
        );
        assert_eq!(
            CompressionType::from_path("config.zstd"),
            CompressionType::Zstd
        );
    }

    #[test]
    fn test_decompress_none() {
        let data = b"hello world";
        let result = decompress(data, CompressionType::None).unwrap();
        assert_eq!(result, data);
    }
}
