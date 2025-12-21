//! Driver GOP (Graphics Output Protocol)

use core::ffi::c_void;

use super::{
    framebuffer::{Framebuffer, FramebufferInfo},
    mode::VideoMode,
    pixel::PixelFormat,
};
use crate::{
    core::error::{BootError, Result, VideoError},
    uefi::BootServices,
};

// Definições de GUID e estruturas UEFI cruas (se não existirem no módulo
// uefi::table) Assumindo que o módulo uefi já expõe interfaces básicas.

pub struct GopDriver<'a> {
    boot_services: &'a BootServices,
    gop_interface: *mut c_void,
}

impl<'a> GopDriver<'a> {
    pub fn new(boot_services: &'a BootServices) -> Result<Self> {
        // FIX: Tipo explícito para ponteiro nulo
        let gop_interface: *mut c_void = core::ptr::null_mut();

        // TODO: Implementar locate_protocol real

        Ok(Self {
            boot_services,
            gop_interface,
        })
    }

    /// Retorna uma lista de todos os modos de vídeo suportados pelo hardware.
    pub fn query_modes(&self) -> Result<impl Iterator<Item = VideoMode>> {
        // GOP->QueryMode() loop
        // Retorna um iterador customizado

        // Placeholder retornando vazio
        Ok(core::iter::empty())
    }

    pub fn set_mode(&mut self, _mode_id: Option<u32>) -> Result<FramebufferInfo> {
        Ok(FramebufferInfo {
            addr:   0xB8000,
            size:   4000,
            width:  80,
            height: 25,
            stride: 80,
            format: PixelFormat::BltOnly,
        })
    }

    /// Obtém acesso direto ao Framebuffer atual.
    /// Requer `&mut self` pois pode alterar o modo de vídeo.
    pub unsafe fn get_framebuffer(&mut self) -> Result<Framebuffer> {
        let info = self.set_mode(None)?;

        // Retorna Framebuffer diretamente (assumindo que Framebuffer::new é infalível
        // ou unsafe)
        Ok(Framebuffer::new(info.addr, info))
    }
}

impl From<u32> for PixelFormat {
    fn from(uefi_fmt: u32) -> Self {
        match uefi_fmt {
            0 => PixelFormat::RgbReserved8Bit,
            1 => PixelFormat::BgrReserved8Bit,
            2 => PixelFormat::Bitmask,
            _ => PixelFormat::BltOnly,
        }
    }
}
