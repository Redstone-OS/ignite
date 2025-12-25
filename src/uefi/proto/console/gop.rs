//! Graphics Output Protocol (GOP)
//!
//! Permite desenhar na tela em modo gráfico de alta resolução.
//! Referência: UEFI Spec 2.10, Seção 12.9

use crate::uefi::base::{Guid, Status};

/// GUID do Protocolo GOP.
pub const GRAPHICS_OUTPUT_PROTOCOL_GUID: Guid = Guid::new(
    0x9042a9de,
    0x23dc,
    0x4a38,
    [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
);

/// Formato de Pixel.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PixelFormat {
    PixelRedGreenBlueReserved8BitPerColor = 0,
    PixelBlueGreenRedReserved8BitPerColor = 1,
    PixelBitMask = 2,
    PixelBltOnly = 3,
    PixelFormatMax = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PixelBitmask {
    pub red_mask:      u32,
    pub green_mask:    u32,
    pub blue_mask:     u32,
    pub reserved_mask: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GraphicsOutputModeInformation {
    pub version:               u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution:   u32,
    pub pixel_format:          PixelFormat,
    pub pixel_information:     PixelBitmask,
    pub pixels_per_scan_line:  u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GraphicsOutputProtocolMode {
    pub max_mode:          u32,
    pub mode:              u32,
    pub info:              *mut GraphicsOutputModeInformation,
    pub size_of_info:      usize,
    pub frame_buffer_base: u64,
    pub frame_buffer_size: usize,
}

/// A Interface do Protocolo GOP.
#[repr(C)]
pub struct GraphicsOutputProtocol {
    pub query_mode: extern "efiapi" fn(
        *mut Self,
        u32,
        *mut usize,
        *mut *mut GraphicsOutputModeInformation,
    ) -> Status,
    pub set_mode:   extern "efiapi" fn(*mut Self, u32) -> Status,
    pub blt: extern "efiapi" fn(
        *mut Self,
        *mut core::ffi::c_void,
        u32,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) -> Status,
    pub mode:       *mut GraphicsOutputProtocolMode,
}

impl GraphicsOutputProtocol {
    pub fn mode_info(&self) -> &GraphicsOutputProtocolMode {
        unsafe { &*self.mode }
    }
}
