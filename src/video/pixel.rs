//! Definição de Formatos de Pixel e Cores
//!
//! O UEFI geralmente usa BGR (Blue-Green-Red) com um byte reservado (Padding).
//! Este arquivo abstrai essas diferenças para que o resto do sistema desenhe
//! cores corretamente.

/// Formatos de pixel suportados pelo hardware gráfico.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PixelFormat {
    /// Vermelho, Verde, Azul, Reservado (8 bits cada).
    RgbReserved8Bit,
    /// Azul, Verde, Vermelho, Reservado (8 bits cada) - Padrão UEFI mais comum.
    BgrReserved8Bit,
    /// Controlado por máscaras de bits específicas (menos comum em hardware
    /// moderno).
    Bitmask,
    /// Formato apenas para Blt (Block Transfer), não suportado diretamente no
    /// frame.
    BltOnly,
}

/// Representa uma cor RGBA independente de hardware.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, // Alpha não é usado na saída direta do UEFI, mas útil para mistura de software.
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}
