//! Theme System

/// Color in RGB format
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
}

/// UI Theme
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub selection:  Color,
    pub comment:    Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::BLACK,
            foreground: Color::WHITE,
            selection:  Color::CYAN,
            comment:    Color {
                r: 128,
                g: 128,
                b: 128,
            },
        }
    }
}
