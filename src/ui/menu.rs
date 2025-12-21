//! Implementação do Menu de Boot
//!
//! Menu interativo para seleção de entradas de boot

use log::info;

use crate::{
    config::types::{BootConfig, MenuEntry},
    error::Result,
};

/// Menu de boot
pub struct BootMenu<'a> {
    config:        &'a BootConfig,
    current_entry: usize,
}

impl<'a> BootMenu<'a> {
    pub fn new(config: &'a BootConfig) -> Self {
        Self {
            config,
            current_entry: config.default_entry.saturating_sub(1),
        }
    }

    /// Exibir menu e aguardar seleção
    pub fn show(&mut self) -> Result<&MenuEntry> {
        info!("═══════════════════════════════════════════════════");
        info!("  Boot Menu - Ignite Bootloader");
        info!("═══════════════════════════════════════════════════");

        for (i, entry) in self.config.entries.iter().enumerate() {
            let marker = if i == self.current_entry { "►" } else { " " };
            info!("{} {}", marker, entry.name);

            if let Some(ref comment) = entry.comment {
                info!("    {}", comment);
            }
        }

        info!("═══════════════════════════════════════════════════");
        info!("Use ↑↓ to navigate, Enter to select, E to edit");

        // TODO: Manipulação real de entrada
        // Por enquanto, retorna a entrada selecionada

        Ok(&self.config.entries[self.current_entry])
    }

    /// Navegar para cima no menu
    pub fn move_up(&mut self) {
        if self.current_entry > 0 {
            self.current_entry -= 1;
        }
    }

    /// Navegar para baixo no menu
    pub fn move_down(&mut self) {
        if self.current_entry < self.config.entries.len() - 1 {
            self.current_entry += 1;
        }
    }

    /// Obter entrada atualmente selecionada
    pub fn current_entry(&self) -> &MenuEntry {
        &self.config.entries[self.current_entry]
    }
}
