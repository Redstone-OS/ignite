//! Implementação de saída de vídeo usando GOP (Graphics Output Protocol)

use uefi::{
    boot::{OpenProtocolAttributes, OpenProtocolParams},
    prelude::*,
    proto::console::gop::GraphicsOutput,
};

use crate::{
    error::{BootError, Result, VideoError},
    types::Framebuffer,
    video::VideoOutput,
};

/// Saída de vídeo usando GOP
pub struct GopVideoOutput<'a> {
    boot_services: &'a BootServices,
    image_handle:  Handle,
    framebuffer:   Option<Framebuffer>,
}

impl<'a> GopVideoOutput<'a> {
    /// Cria uma nova instância de GopVideoOutput
    pub fn new(boot_services: &'a BootServices, image_handle: Handle) -> Self {
        Self {
            boot_services,
            image_handle,
            framebuffer: None,
        }
    }
}

impl<'a> VideoOutput for GopVideoOutput<'a> {
    fn initialize(&mut self) -> Result<()> {
        // Obter handle do protocolo GOP
        let gop_handle = self
            .boot_services
            .get_handle_for_protocol::<GraphicsOutput>()
            .map_err(|_| BootError::Video(VideoError::NoGopHandle))?;

        // Abrir protocolo GOP
        let mut gop = unsafe {
            self.boot_services
                .open_protocol::<GraphicsOutput>(
                    OpenProtocolParams {
                        handle:     gop_handle,
                        agent:      self.image_handle,
                        controller: None,
                    },
                    OpenProtocolAttributes::GetProtocol,
                )
                .map_err(|_| BootError::Video(VideoError::GopOpenFailed))?
        };

        // Obter informações do modo atual
        let mode_info = gop.current_mode_info();
        let mut frame_buffer = gop.frame_buffer();

        // Criar estrutura Framebuffer
        let fb = Framebuffer {
            ptr:                   frame_buffer.as_mut_ptr() as u64,
            size:                  frame_buffer.size(),
            horizontal_resolution: mode_info.resolution().0,
            vertical_resolution:   mode_info.resolution().1,
            stride:                mode_info.stride(),
        };

        self.framebuffer = Some(fb);

        log::info!(
            "GOP inicializado: {}x{} (stride: {})",
            fb.horizontal_resolution,
            fb.vertical_resolution,
            fb.stride
        );

        Ok(())
    }

    fn get_framebuffer(&self) -> Framebuffer {
        self.framebuffer
            .expect("GOP não foi inicializado. Chame initialize() primeiro.")
    }

    fn set_mode(&mut self, _width: usize, _height: usize) -> Result<()> {
        // TODO: Implementar mudança de modo de vídeo
        // Por enquanto, usa o modo atual
        Err(BootError::Video(VideoError::UnsupportedMode))
    }
}
