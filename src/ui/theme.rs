//! Definições de Tema e Cores
//!
//! Padroniza a aparência da interface gráfica.

use crate::video::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background:  Color,
    pub foreground:  Color,
    pub highlight:   Color,
    pub comment:     Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background:  Color::new(30, 30, 46), // Dark Blue/Grey (Catppuccin base-ish)
            foreground:  Color::new(205, 214, 244), // White-ish
            highlight:   Color::new(137, 180, 250), // Blue
            comment:     Color::new(108, 112, 134), // Grey
            selected_bg: Color::new(49, 50, 68), // Surface0
            selected_fg: Color::new(250, 179, 135), // Peach/Orange
        }
    }
}
