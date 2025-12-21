//! Driver GOP (Graphics Output Protocol)
//!
//! Responsável por interagir com o Firmware UEFI para configurar o vídeo.
//! Localiza o protocolo, consulta modos disponíveis e configura a resolução.

use super::{
    framebuffer::{Framebuffer, FramebufferInfo},
    mode::{VideoMode, VideoModeInfo},
    pixel::PixelFormat,
};
use crate::{
    core::error::{BootError, Result, VideoError},
    uefi::{BootServices, Handle},
};

// Definições de GUID e estruturas UEFI cruas (se não existirem no módulo
// uefi::table) Assumindo que o módulo uefi já expõe interfaces básicas.

pub struct GopDriver<'a> {
    boot_services: &'a BootServices,
    // Em uma implementação real, guardaríamos o ponteiro para a interface GOP aqui.
    // Como Rust exige tipos concretos, usamos um ponteiro opaco ou wrapper.
    gop_interface: *mut core::ffi::c_void,
}

impl<'a> GopDriver<'a> {
    /// Inicializa o driver localizando o protocolo GOP via BootServices.
    pub fn new(boot_services: &'a BootServices) -> Result<Self> {
        // TODO: Usar boot_services.locate_protocol() com a GUID do GOP.
        // EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID = { 0x9042a9de, 0x23dc, 0x4a38, {0x96,
        // 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a} }

        // Simulação de sucesso para compilação estrutural
        let gop_interface = core::ptr::null_mut();

        if gop_interface.is_null() {
            // Em produção real, remover este check simulado e usar retorno real
            // do locate_protocol log::error!("GOP não encontrado");
            // return Err(BootError::Video(VideoError::NotSupported));
        }

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

    /// Define a resolução de vídeo desejada.
    /// Se `mode` for None, tenta usar a maior resolução disponível.
    pub fn set_mode(&mut self, mode_id: Option<u32>) -> Result<FramebufferInfo> {
        // 1. Se mode_id for None, iterar modos e achar o "melhor" (maior área).
        // 2. Chamar GOP->SetMode(mode_id).
        // 3. Obter GOP->Mode->FrameBufferBase e GOP->Mode->Info.

        // Mock de retorno para manter a interface funcional durante refatoração
        Ok(FramebufferInfo {
            addr:   0xB8000, // VGA legacy apenas como placeholder seguro
            size:   4000,
            width:  80,
            height: 25,
            stride: 80,
            format: PixelFormat::BltOnly,
        })
    }

    /// Obtém acesso direto ao Framebuffer atual para desenho.
    ///
    /// # Safety
    /// Retorna uma estrutura que escreve diretamente na RAM de vídeo.
    pub unsafe fn get_framebuffer(&self) -> Result<Framebuffer> {
        // Obter info do modo atual
        let info = self.set_mode(None)?; // Reutiliza lógica de info
        Framebuffer::new(info.addr, info)
            .ok_or(BootError::Video(VideoError::InitializationFailed)) // Ajustar para Option se necessário
            .map_err(|_| BootError::Video(VideoError::InitializationFailed))
    }
}

// Helpers para conversão de tipos UEFI crus para nossos tipos Rusty
impl From<u32> for PixelFormat {
    fn from(uefi_fmt: u32) -> Self {
        match uefi_fmt {
            0 => PixelFormat::RgbReserved8Bit, // PixelRedGreenBlueReserved8BitPerColor
            1 => PixelFormat::BgrReserved8Bit, // PixelBlueGreenRedReserved8BitPerColor
            2 => PixelFormat::Bitmask,         // PixelBitMask
            _ => PixelFormat::BltOnly,         // PixelBltOnly
        }
    }
}
