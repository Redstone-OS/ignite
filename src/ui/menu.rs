//! Menu de Boot Gráfico
//!
//! Renderiza as opções de boot e gerencia a navegação.
//! Protegido contra resoluções extremas ou listas vazias.

use super::{
    graphics::GraphicsContext,
    input::{InputManager, Key},
    theme::Theme,
};
use crate::{
    config::{BootConfig, Entry},
    core::handoff::FramebufferInfo,
    video::Color,
};

pub struct Menu<'a> {
    config:         &'a BootConfig,
    theme:          Theme,
    selected_index: usize,
    input:          InputManager,
}

impl<'a> Menu<'a> {
    pub fn new(config: &'a BootConfig) -> Self {
        // Garante que o índice selecionado é válido, mesmo se a lista mudou
        let max_index = if config.entries.is_empty() {
            0
        } else {
            config.entries.len() - 1
        };
        let selected_index = config.default_entry_idx.min(max_index);

        Self {
            config,
            theme: Theme::default(),
            selected_index,
            input: InputManager::new(),
        }
    }

    /// Executa o loop do menu.
    pub unsafe fn run(&mut self, fb_ptr: u64, fb_info: FramebufferInfo) -> &'a Entry {
        let mut ctx = GraphicsContext::new(fb_ptr, fb_info);

        // Se não houver entradas (o que o Default previne, mas por segurança), trava.
        if self.config.entries.is_empty() {
            crate::println!("ERRO CRITICO: Nenhuma entrada de boot disponivel.");
            loop {
                crate::arch::hlt();
            }
        }

        loop {
            self.draw(&mut ctx);

            match self.input.wait_for_key() {
                Key::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.config.entries.len() - 1;
                    }
                },
                Key::Down => {
                    if self.selected_index < self.config.entries.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                },
                Key::Enter => {
                    return &self.config.entries[self.selected_index];
                },
                _ => {}, // Ignorar outras teclas
            }
        }
    }

    fn draw(&self, ctx: &mut GraphicsContext) {
        ctx.clear(self.theme.background);

        let width = ctx.width();
        let height = ctx.height();

        // Proteção contra telas muito pequenas (Evita panic de overflow)
        if width < 200 || height < 150 {
            return;
        }

        // --- Cabeçalho ---
        let title = "Ignite Bootloader";
        let title_len_px = title.len() as u32 * 8;
        // Centralização segura
        let title_x = if width > title_len_px {
            (width - title_len_px) / 2
        } else {
            0
        };
        ctx.draw_string(title_x, 30, title, self.theme.highlight, None);

        // --- Lista de Entradas ---
        let start_y = 100;
        let line_height = 20;

        for (i, entry) in self.config.entries.iter().enumerate() {
            let y = start_y + (i as u32 * line_height);
            // Evita desenhar fora da tela verticalmente
            if y + line_height > height {
                break;
            }

            let is_selected = i == self.selected_index;

            let fg = if is_selected {
                self.theme.selected_fg
            } else {
                self.theme.foreground
            };

            // Fundo da seleção (com cálculo de largura seguro)
            if is_selected {
                let rect_x = 50;
                // CORREÇÃO: Verifica se width permite subtrair 100
                let rect_w = if width > 100 {
                    width - 100
                } else {
                    width.saturating_sub(rect_x)
                };

                if rect_w > 0 {
                    ctx.fill_rect(rect_x, y - 2, rect_w, 18, self.theme.selected_bg);
                }
            }

            let prefix = if is_selected { "> " } else { "  " };
            ctx.draw_string(60, y, prefix, fg, None);
            ctx.draw_string(80, y, &entry.name, fg, None);
        }

        // --- Rodapé ---
        let footer = "Setas: Navegar | Enter: Selecionar";
        let footer_len_px = footer.len() as u32 * 8;
        let footer_x = if width > footer_len_px {
            (width - footer_len_px) / 2
        } else {
            0
        };

        if height > 30 {
            ctx.draw_string(footer_x, height - 30, footer, self.theme.comment, None);
        }
    }
}
