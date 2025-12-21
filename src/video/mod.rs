//! Módulo de configuração de vídeo
//!
//! Responsável por inicializar e configurar a saída de vídeo via GOP (Graphics
//! Output Protocol)

pub mod gop;

pub use gop::GopVideoOutput;

use crate::{error::Result, types::Framebuffer};

/// Trait para abstração de saída de vídeo
pub trait VideoOutput {
    /// Inicializa a saída de vídeo
    fn initialize(&mut self) -> Result<()>;

    /// Define o modo de vídeo (resolução) e retorna Framebuffer
    fn set_mode(&mut self, width: usize, height: usize) -> Result<Framebuffer>;
}
