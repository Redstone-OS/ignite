//! Gerenciamento de Modos de Vídeo (Resoluções)
//!
//! Encapsula as informações retornadas pelo GOP sobre o que o monitor suporta.

use super::pixel::PixelFormat;

/// Informações sobre um modo de vídeo específico.
#[derive(Debug, Clone, Copy)]
pub struct VideoModeInfo {
    /// Largura visível em pixels.
    pub width:  usize,
    /// Altura visível em pixels.
    pub height: usize,
    /// Pixels por linha de scan (Stride).
    /// Frequentemente maior que `width` devido ao alinhamento de hardware.
    pub stride: usize,
    /// Formato dos pixels.
    pub format: PixelFormat,
}

/// Identificador de um modo de vídeo.
#[derive(Debug, Clone, Copy)]
pub struct VideoMode {
    /// ID interno do UEFI para este modo.
    pub id:   u32,
    /// Detalhes do modo.
    pub info: VideoModeInfo,
}

impl VideoMode {
    /// Retorna o tamanho do buffer necessário para este modo em bytes.
    pub fn framebuffer_size_bytes(&self) -> usize {
        self.info.stride * self.info.height * 4 // 4 bytes por pixel (32-bit color)
    }
}
