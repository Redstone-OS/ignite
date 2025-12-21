//! Implementação de saída de vídeo usando GOP (Graphics Output Protocol)

use crate::{
    core::{
        error::{BootError, Result, VideoError},
        types::Framebuffer,
    },
    uefi::{
        BootServices, Handle,
        proto::console::gop::{GRAPHICS_OUTPUT_PROTOCOL_GUID, GraphicsOutputProtocol},
        table::boot::OPEN_PROTOCOL_EXCLUSIVE,
    },
    video::VideoOutput,
};

/// Saída de vídeo usando GOP
pub struct GopVideoOutput<'a> {
    boot_services: &'a BootServices,
    image_handle:  Handle,
}

impl<'a> GopVideoOutput<'a> {
    pub fn new(boot_services: &'a BootServices, image_handle: Handle) -> Self {
        Self {
            boot_services,
            image_handle,
        }
    }

    /// Obtém o framebuffer do GOP
    pub fn get_framebuffer(&self) -> Framebuffer {
        // TODO: Implementar quando GOP parser estiver funcionando
        Framebuffer {
            ptr: 0,
            size: 0,
            horizontal_resolution: 0,
            vertical_resolution:   0,
            stride:                0,
        }
    }
}

impl<'a> VideoOutput for GopVideoOutput<'a> {
    fn initialize(&mut self) -> Result<()> {
        // TODO: Implementar inicialização GOP usando nossa camada UEFI pura
        // Requer: locate_protocol e open_protocol nas BootServices
        log::warn!("GOP initialization not yet fully implemented");
        Err(BootError::Video(VideoError::InitializationFailed))
    }

    fn set_mode(&mut self, _width: usize, _height: usize) -> Result<Framebuffer> {
        // TODO: Implementar set_mode quando GOP estiver funcionando
        log::warn!("GOP set_mode not yet implemented");
        Err(BootError::Video(VideoError::UnsupportedMode))
    }
}
