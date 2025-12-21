//! Driver GOP (Graphics Output Protocol)
//!
//! Interage com o firmware UEFI para configurar vídeo e acessar framebuffer
//! nativo.

use core::ffi::c_void;

use super::{
    framebuffer::{Framebuffer, FramebufferInfo},
    mode::VideoMode,
    pixel::PixelFormat,
};
use crate::{
    core::error::{BootError, Result, VideoError},
    uefi::{BootServices, Handle},
};

// GUID do Protocolo GOP: {9042A9DE-23DC-4A38-96FB-7ADED080516A}
pub const GRAPHICS_OUTPUT_PROTOCOL_GUID: crate::uefi::base::Guid = crate::uefi::base::Guid::new(
    0x9042a9de,
    0x23dc,
    0x4a38,
    [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
);

pub struct GopDriver<'a> {
    boot_services: &'a BootServices,
    gop_interface: *mut crate::uefi::proto::console::gop::GraphicsOutputProtocol,
}

impl<'a> GopDriver<'a> {
    pub fn new(boot_services: &'a BootServices) -> Result<Self> {
        let gop_void_ptr = boot_services
            .locate_protocol(&GRAPHICS_OUTPUT_PROTOCOL_GUID)
            .map_err(|_| BootError::Video(VideoError::GopNotSupported))?;

        let gop_interface =
            gop_void_ptr as *mut crate::uefi::proto::console::gop::GraphicsOutputProtocol;

        Ok(Self {
            boot_services,
            gop_interface,
        })
    }

    fn get_current_mode_info(&self) -> Result<FramebufferInfo> {
        unsafe {
            let gop = &*self.gop_interface;
            let mode = &*gop.mode;
            let info = &*mode.info;

            Ok(FramebufferInfo {
                addr: mode.frame_buffer_base,
                size: mode.frame_buffer_size,
                width: info.horizontal_resolution,
                height: info.vertical_resolution,
                stride: info.pixels_per_scan_line,
                format: match info.pixel_format {
                    crate::uefi::proto::console::gop::PixelFormat::PixelRedGreenBlueReserved8BitPerColor => PixelFormat::RgbReserved8Bit,
                    crate::uefi::proto::console::gop::PixelFormat::PixelBlueGreenRedReserved8BitPerColor => PixelFormat::BgrReserved8Bit,
                    crate::uefi::proto::console::gop::PixelFormat::PixelBitMask => PixelFormat::Bitmask,
                    _ => PixelFormat::BltOnly,
                },
            })
        }
    }

    pub fn query_modes(&self) -> Result<impl Iterator<Item = VideoMode>> {
        Ok(core::iter::empty())
    }

    pub fn set_mode(&mut self, _mode_id: Option<u32>) -> Result<FramebufferInfo> {
        self.get_current_mode_info()
    }

    /// # Safety
    /// Retorna uma estrutura que escreve diretamente na VRAM.
    pub unsafe fn get_framebuffer(&mut self) -> Result<Framebuffer> {
        let info = self.get_current_mode_info()?;

        if info.addr == 0 || info.width == 0 || info.height == 0 {
            return Err(BootError::Video(VideoError::InitializationFailed));
        }

        Ok(Framebuffer::new(info.addr, info))
    }
}
