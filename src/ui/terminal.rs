//! Terminal Gráfico
//!
//! Renderização de texto no framebuffer

use alloc::vec::Vec;

/// Terminal gráfico para renderização de texto
pub struct GraphicalTerminal {
    width:       u32,
    height:      u32,
    rows:        u32,
    cols:        u32,
    current_row: u32,
    current_col: u32,
}

impl GraphicalTerminal {
    pub fn new(width: u32, height: u32) -> Self {
        // Assume fonte 8x16
        let cols = width / 8;
        let rows = height / 16;

        Self {
            width,
            height,
            rows,
            cols,
            current_row: 0,
            current_col: 0,
        }
    }

    /// Escrever um caractere
    pub fn putchar(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.current_col = 0,
            _ => {
                // TODO: Renderizar caractere no framebuffer
                self.current_col += 1;
                if self.current_col >= self.cols {
                    self.newline();
                }
            },
        }
    }

    /// Escrever uma string
    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.putchar(c);
        }
    }

    fn newline(&mut self) {
        self.current_col = 0;
        self.current_row += 1;
        if self.current_row >= self.rows {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        // TODO: Implementar rolagem (scrolling)
        self.current_row = self.rows - 1;
    }

    /// Limpar tela
    pub fn clear(&mut self) {
        // TODO: Limpar framebuffer
        self.current_row = 0;
        self.current_col = 0;
    }
}
