//! Menu de Boot Gráfico
//!
//! Renderiza as opções de boot e gerencia a navegação do usuário.

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
        Self {
            config,
            theme: Theme::default(),
            selected_index: config
                .default_entry_idx
                .min(config.entries.len().saturating_sub(1)),
            input: InputManager::new(),
        }
    }

    /// Executa o loop do menu até o usuário selecionar algo.
    /// Retorna a entrada escolhida.
    ///
    /// # Safety
    /// Requer acesso direto ao buffer de vídeo.
    pub unsafe fn run(&mut self, fb_ptr: u64, fb_info: FramebufferInfo) -> &'a Entry {
        let mut ctx = GraphicsContext::new(fb_ptr, fb_info);

        // Se houver timeout configurado, tratar aqui (lógica simplificada para
        // brevidade) Em produção: loop com verificação de tempo + input poll.

        // Loop principal de UI
        loop {
            self.draw(&mut ctx);

            match self.input.wait_for_key() {
                Key::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.config.entries.len() - 1; // Wrap around
                    }
                },
                Key::Down => {
                    if self.selected_index < self.config.entries.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0; // Wrap around
                    }
                },
                Key::Enter => {
                    return &self.config.entries[self.selected_index];
                },
                _ => {}, // Ignorar outros
            }
        }
    }

    fn draw(&self, ctx: &mut GraphicsContext) {
        // Fundo
        ctx.clear(self.theme.background);

        // Cabeçalho
        let title = "Ignite Bootloader";
        let title_x = (ctx.width() - (title.len() as u32 * 8)) / 2;
        ctx.draw_string(title_x, 30, title, self.theme.highlight, None);

        // Lista de Entradas
        let start_y = 100;
        let line_height = 20;

        for (i, entry) in self.config.entries.iter().enumerate() {
            let y = start_y + (i as u32 * line_height);
            let is_selected = i == self.selected_index;

            let fg = if is_selected {
                self.theme.selected_fg
            } else {
                self.theme.foreground
            };
            let bg = if is_selected {
                Some(self.theme.selected_bg)
            } else {
                None
            };

            // Indicador de seleção
            let prefix = if is_selected { "> " } else { "  " };

            // Desenhar linha (fundo da seleção primeiro se necessário)
            if is_selected {
                ctx.fill_rect(50, y - 2, ctx.width() - 100, 18, self.theme.selected_bg);
            }

            ctx.draw_string(60, y, prefix, fg, None);
            ctx.draw_string(80, y, &entry.name, fg, None);
        }

        // Rodapé
        let footer = "Use as setas para navegar, Enter para selecionar.";
        let footer_x = (ctx.width() - (footer.len() as u32 * 8)) / 2;
        ctx.draw_string(
            footer_x,
            ctx.height() - 30,
            footer,
            self.theme.comment,
            None,
        );
    }
}
