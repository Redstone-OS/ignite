//! Menu de boot interativo
//!
//! Exibe menu para seleção de sistema operacional

use crate::config::{BootConfig, OsEntry, OsList};
use log::info;

/// Menu de boot
pub struct BootMenu;

impl BootMenu {
    /// Exibe menu e retorna OS selecionado
    ///
    /// TODO: Implementar interface interativa
    /// - Listar sistemas operacionais disponíveis
    /// - Permitir navegação com setas
    /// - Permitir seleção com Enter
    /// - Implementar timeout visual
    /// - Destacar opção padrão
    ///
    /// # Argumentos
    /// * `config` - Configuração do bootloader
    /// * `os_list` - Lista de sistemas operacionais
    ///
    /// # Retorna
    /// Entrada de OS selecionada
    pub fn show(config: &BootConfig, os_list: &OsList) -> &OsEntry {
        info!("╔══════════════════════════════════════════════════╗");
        info!("║           Ignite Boot Menu v0.2.0                ║");
        info!("╠══════════════════════════════════════════════════╣");
        
        // TODO: Listar opções
        for (i, entry) in os_list.entries.iter().enumerate() {
            info!("║  {}. {}                                      ║", i + 1, entry.name);
        }
        
        info!("╠══════════════════════════════════════════════════╣");
        info!("║  Use ↑↓ para navegar, Enter para selecionar     ║");
        info!("║  Timeout: {} segundos                           ║", config.timeout);
        info!("╚══════════════════════════════════════════════════╝");

        // TODO: Implementar leitura de input e seleção
        // Por enquanto, retorna primeira opção (Redstone)
        &os_list.entries[0]
    }

    /// Mostra hint de tecla de configuração
    pub fn show_config_hint() {
        info!("Pressione 'C' para configuração");
        // TODO: Posicionar no canto inferior esquerdo usando GOP
    }
}
