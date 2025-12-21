//! Subsistema de Vídeo do Bootloader
//!
//! Fornece gráficos de alta resolução via UEFI GOP (Graphics Output Protocol).
//! Permite desenhar na tela durante o boot e prepara o Framebuffer para o
//! Kernel.

pub mod framebuffer;
pub mod gop;
pub mod mode;
pub mod pixel;

// Re-exportações para facilitar o uso no `main.rs`
pub use framebuffer::{Framebuffer, FramebufferInfo};
pub use gop::GopDriver;
pub use mode::{VideoMode, VideoModeInfo};
pub use pixel::{Color, PixelFormat};

use crate::core::error::Result;

/// Inicializa o vídeo na melhor resolução possível e limpa a tela.
/// Retorna o driver GOP e o Framebuffer ativo.
pub fn init_video(
    boot_services: &crate::uefi::BootServices,
) -> Result<(GopDriver, FramebufferInfo)> {
    let mut driver = GopDriver::new(boot_services)?;

    // Auto-detecta e configura a melhor resolução (geralmente nativa do monitor)
    let fb_info = driver.set_mode(None)?;

    // (Opcional) Limpar a tela ou desenhar logo aqui
    // let mut fb = unsafe { driver.get_framebuffer()? };
    // fb.clear(Color::BLACK);

    Ok((driver, fb_info))
}
