//! # GOP Video Subsystem
//!
//! O subsistema `video` é responsável por tirar o computador da "Idade das
//! Trevas" (Modo Texto/VGA) e habilitar gráficos de alta resolução via UEFI GOP
//! (Graphics Output Protocol).
//!
//! ## 🎯 Responsabilidades
//! 1. **Handshake GOP:** Encontrar o protocolo gráfico firmware.
//! 2. **Mode Switch:** Configurar resolução nativa do monitor (ou fallback
//!    seguro).
//! 3. **Raw Access:** Expor o Framebuffer linear para que a UI do Ignite e
//!    depois o Kernel possam desenhar pixels.
//!
//! ## 🔍 Análise Crítica (Kernel Engineer's View)
//!
//! ### ✅ Pontos Fortes
//! - **Abstração Limpa:** Separa a lógica "suja" do UEFI (`gop.rs`) da
//!   representação agnóstica (`framebuffer.rs`).
//! - **Handoff Friendly:** As structs `FramebufferInfo` são desenhadas para
//!   serem passadas para o Kernel sem dependência de UEFI.
//!
//! ### ⚠️ Pontos de Atenção (Riscos e Dívida)
//! - **Hardcoded Auto-Detect:** A função `init_video` ignora preferências de
//!   resolução. Se o monitor reportar EDID errado, ficamos presos em resolução
//!   ruim.
//!   - *Correção:* Permitir override via `ignite.cfg` (ex: `video_mode =
//!     "1920x1080"`).
//! - **Performance de Escrita:** Desenhar pixel a pixel no Framebuffer UEFI é
//!   lento (uncached write-combining memory).
//!   - *Mitigação:* A UI deve usar Double Buffering em RAM e fazer *Dirty Rect
//!     Blit*.
//!
//! ## 🛠️ TODOs e Roadmap
//! - [ ] **TODO: (Config)** Implementar seleção de resolução baseada em
//!   `ignite.cfg`.
//! - [ ] **TODO: (Driver)** Analisar suporte a múltiplos monitores (GOP
//!   geralmente só expõe o primário).

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
) -> Result<(GopDriver<'_>, FramebufferInfo)> {
    let mut driver = GopDriver::new(boot_services)?;

    // Auto-detecta e configura a melhor resolução (geralmente nativa do monitor)
    let fb_info = driver.set_mode(None)?;

    // (Opcional) Limpar a tela ou desenhar logo aqui
    // let mut fb = unsafe { driver.get_framebuffer()? };
    // fb.clear(Color::BLACK);

    Ok((driver, fb_info))
}
