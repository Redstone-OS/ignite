//! Graphics Output Protocol (GOP)
//!
//! Referência: UEFI Spec 2.10, Seção 12.9

use crate::uefi::base::*;

/// Graphics Output Protocol GUID
pub const GRAPHICS_OUTPUT_PROTOCOL_GUID: Guid = Guid::new(
    0x9042a9de,
    0x23dc,
    0x4a38,
    [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
);

/// Pixel Format
///
/// Spec: 12.9.2
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// Pixels are 32-bit: Red[31:24], Green[23:16], Blue[15:8], Reserved[7:0]
    PixelRedGreenBlueReserved8BitPerColor = 0,
    /// Pixels are 32-bit: Blue[31:24], Green[23:16], Red[15:8], Reserved[7:0]
    PixelBlueGreenRedReserved8BitPerColor = 1,
    /// Pixels use bitmask
    PixelBitMask = 2,
    /// Frame buffer não é acessível diretamente, apenas Blt
    PixelBltOnly = 3,
    PixelFormatMax = 4,
}

/// Pixel Bitmask
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PixelBitmask {
    pub red_mask:      u32,
    pub green_mask:    u32,
    pub blue_mask:     u32,
    pub reserved_mask: u32,
}

/// Graphics Output Mode Information
///
/// Spec: 12.9.3
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GraphicsOutputModeInformation {
    /// Versão da estrutura
    pub version:               u32,
    /// Resolução horizontal em pixels
    pub horizontal_resolution: u32,
    /// Resolução vertical em pixels
    pub vertical_resolution:   u32,
    /// Formato dos pixels
    pub pixel_format:          PixelFormat,
    /// Informação sobre bitmask (válido apenas se PixelBitMask)
    pub pixel_information:     PixelBitmask,
    /// Pixels por linha de scan
    pub pixels_per_scan_line:  u32,
}

/// Graphics Output Protocol Mode
///
/// Spec: 12.9.4
#[repr(C)]
pub struct GraphicsOutputProtocolMode {
    /// Número máximo de modos suportados
    pub max_mode:          u32,
    /// Modo atual
    pub mode:              u32,
    /// Ponteiro para informações do modo atual
    pub info:              *mut GraphicsOutputModeInformation,
    /// Tamanho da estrutura de informação
    pub size_of_info:      usize,
    /// Endereço base do framebuffer
    pub frame_buffer_base: u64,
    /// Tamanho do framebuffer em bytes
    pub frame_buffer_size: usize,
}

/// Blt Operation
#[repr(u32)]
pub enum BltOperation {
    /// Preencher retângulo com cor
    BltVideoFill = 0,
    /// Copiar de buffer para vídeo
    BltVideoToBltBuffer = 1,
    /// Copiar de vídeo para buffer
    BltBufferToVideo = 2,
    /// Copiar de área de vídeo para outra
    BltVideoToVideo = 3,
    GraphicsOutputBltOperationMax = 4,
}

/// Blt Pixel
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BltPixel {
    pub blue:     u8,
    pub green:    u8,
    pub red:      u8,
    pub reserved: u8,
}

/// Graphics Output Protocol
///
/// Spec: 12.9.1
#[repr(C)]
pub struct GraphicsOutputProtocol {
    /// Query informações sobre modo específico
    pub query_mode: extern "efiapi" fn(
        *mut Self,
        u32,                                     // ModeNumber
        *mut usize,                              // SizeOfInfo
        *mut *mut GraphicsOutputModeInformation, // Info
    ) -> Status,

    /// Define o modo de vídeo
    pub set_mode: extern "efiapi" fn(
        *mut Self,
        u32, // ModeNumber
    ) -> Status,

    /// Operação Blt (Block Transfer)
    pub blt: extern "efiapi" fn(
        *mut Self,
        *mut BltPixel, // BltBuffer
        BltOperation,  // BltOperation
        usize,         // SourceX
        usize,         // SourceY
        usize,         // DestinationX
        usize,         // DestinationY
        usize,         // Width
        usize,         // Height
        usize,         // Delta
    ) -> Status,

    /// Ponteiro para informações do modo atual
    pub mode: *mut GraphicsOutputProtocolMode,
}
