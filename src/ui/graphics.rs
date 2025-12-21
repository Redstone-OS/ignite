//! Motor Gráfico Básico (Software Rendering)
//!
//! Fornece primitivas de desenho (retângulos, texto, limpeza) sobre o
//! Framebuffer. Abstrai o formato de pixel (RGB/BGR) e lida com clipping
//! básico.

use super::font::BitFont;
use crate::{
    core::handoff::FramebufferInfo,
    video::{Color, Framebuffer, PixelFormat},
};

/// Contexto gráfico para desenho.
pub struct GraphicsContext<'a> {
    buffer: &'a mut [u8],
    info:   FramebufferInfo,
    font:   BitFont,
}

impl<'a> GraphicsContext<'a> {
    /// Cria um novo contexto gráfico sobre um buffer de memória de vídeo bruto.
    ///
    /// # Safety
    /// O chamador deve garantir que `buffer_ptr` aponta para uma região válida
    /// de VRAM com o tamanho especificado em `info`.
    pub unsafe fn new(buffer_ptr: u64, info: FramebufferInfo) -> Self {
        let buffer = core::slice::from_raw_parts_mut(buffer_ptr as *mut u8, info.size);

        Self {
            buffer,
            info,
            font: BitFont::default(), // Fonte VGA 8x16 embutida
        }
    }

    /// Limpa a tela com uma cor.
    pub fn clear(&mut self, color: Color) {
        // Otimização: preencher linha a linha ou usar memset se a cor for preta
        for y in 0..self.info.height {
            for x in 0..self.info.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    /// Desenha um único pixel.
    #[inline(always)]
    pub fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_offset = (y as usize * self.info.stride as usize) + x as usize;
        let byte_offset = pixel_offset * 4; // Assumindo 32bpp (4 bytes)

        // Verifica limites do buffer
        if byte_offset + 3 >= self.buffer.len() {
            return;
        }

        // Mapeia componentes de cor baseado no formato do vídeo
        let (r, g, b) = match self.info.format {
            // PixelFormat::RgbReserved8BitPerColor
            0 => (color.r, color.g, color.b),
            // PixelFormat::BgrReserved8BitPerColor (Padrão UEFI)
            1 => (color.b, color.g, color.r),
            // Fallback para BGR
            _ => (color.b, color.g, color.r),
        };

        // Escrita direta na VRAM
        self.buffer[byte_offset] = b;
        self.buffer[byte_offset + 1] = g;
        self.buffer[byte_offset + 2] = r;
        // self.buffer[byte_offset + 3] = 0; // Padding/Alpha (ignorado)
    }

    /// Desenha um retângulo preenchido.
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for dy in 0..h {
            for dx in 0..w {
                self.put_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Desenha um caractere usando a fonte embutida.
    pub fn draw_char(&mut self, x: u32, y: u32, c: char, fg: Color, bg: Option<Color>) {
        let glyph = self.font.get_glyph(c);

        for (row_idx, row_byte) in glyph.iter().enumerate() {
            for bit_idx in 0..8 {
                let is_set = (row_byte >> (7 - bit_idx)) & 1 == 1;

                let px = x + bit_idx;
                let py = y + row_idx as u32;

                if is_set {
                    self.put_pixel(px, py, fg);
                } else if let Some(bg_color) = bg {
                    self.put_pixel(px, py, bg_color);
                }
            }
        }
    }

    /// Escreve uma string na posição especificada.
    pub fn draw_string(&mut self, x: u32, y: u32, text: &str, fg: Color, bg: Option<Color>) {
        let mut cur_x = x;
        for c in text.chars() {
            self.draw_char(cur_x, y, c, fg, bg);
            cur_x += 8; // Largura da fonte
        }
    }

    /// Retorna a largura da tela.
    pub fn width(&self) -> u32 {
        self.info.width
    }

    /// Retorna a altura da tela.
    pub fn height(&self) -> u32 {
        self.info.height
    }
}
