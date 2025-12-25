//! Abstração do Framebuffer Linear
//!
//! Representa a região da memória de vídeo (VRAM) onde os pixels são
//! desenhados. Esta estrutura é projetada para ser serializável e enviada ao
//! Kernel via `BootInfo`.


use super::pixel::{Color, PixelFormat};

/// Informações cruas do Framebuffer para Handoff (compatível com C).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    /// Endereço físico do início da memória de vídeo.
    pub addr:   u64,
    /// Tamanho total da memória de vídeo em bytes.
    pub size:   usize,
    /// Largura em pixels.
    pub width:  u32,
    /// Altura em pixels.
    pub height: u32,
    /// Pixels por linha (pode incluir padding invisível).
    pub stride: u32,
    /// Formato dos pixels.
    pub format: PixelFormat,
}

/// Um wrapper seguro em torno da VRAM para operações de desenho no Bootloader.
pub struct Framebuffer<'a> {
    base_addr: *mut u8,
    info:      FramebufferInfo,
    _phantom:  core::marker::PhantomData<&'a mut [u8]>,
}

impl<'a> Framebuffer<'a> {
    /// Cria uma nova interface de framebuffer a partir de informações brutas.
    ///
    /// # Safety
    /// O chamador deve garantir que `base_addr` e `size` são válidos e
    /// mapeados.
    pub unsafe fn new(base_addr: u64, info: FramebufferInfo) -> Self {
        Self {
            base_addr: base_addr as *mut u8,
            info,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Preenche a tela inteira com uma cor.
    pub fn clear(&mut self, color: Color) {
        // Otimização: Se for preto/branco, podemos usar memset rápido
        // Aqui usamos a implementação pixel-a-pixel para correção garantida
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.draw_pixel(x, y, color);
            }
        }
    }

    /// Desenha um único pixel.
    #[inline(always)]
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_offset = (y as usize * self.info.stride as usize) + x as usize;
        let byte_offset = pixel_offset * 4; // 4 bytes por pixel

        unsafe {
            let ptr = self.base_addr.add(byte_offset);

            // Escreve os bytes na ordem correta baseada no formato
            match self.info.format {
                PixelFormat::RgbReserved8Bit => {
                    ptr.add(0).write(color.r);
                    ptr.add(1).write(color.g);
                    ptr.add(2).write(color.b);
                },
                PixelFormat::BgrReserved8Bit => {
                    ptr.add(0).write(color.b);
                    ptr.add(1).write(color.g);
                    ptr.add(2).write(color.r);
                },
                _ => {
                    // Fallback genérico ou Bitmask complexo omitido para brevidade
                    ptr.add(0).write(color.b);
                    ptr.add(1).write(color.g);
                    ptr.add(2).write(color.r);
                },
            }
        }
    }

    /// Retorna as informações para passar ao kernel.
    pub fn info(&self) -> FramebufferInfo {
        self.info
    }
}
