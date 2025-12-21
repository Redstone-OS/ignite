//! Implementação do Menu de Boot Interativo
//!
//! Menu estilo GRUB com suporte a:
//! - Detecção de tecla M durante countdown
//! - Navegação com setas
//! - Seleção por Enter ou número direto
//! - ESC para cancelar

use log::info;
use uefi::{
    proto::console::text::{Key, ScanCode},
    table::boot::BootServices,
};

use crate::{
    config::types::{BootConfig, MenuEntry},
    constants::{boot::MENU_TRIGGER_KEY, ui::*},
    error::Result,
    types::Framebuffer,
};

/// Menu de boot interativo
pub struct BootMenu<'a> {
    boot_services: &'a BootServices,
    config:        &'a BootConfig,
    current_entry: usize,
    framebuffer:   Option<Framebuffer>,
}

impl<'a> BootMenu<'a> {
    /// Cria um novo menu de boot
    ///
    /// # Argumentos
    /// * `boot_services` - Serviços de boot UEFI
    /// * `config` - Configuração de boot
    pub fn new(boot_services: &'a BootServices, config: &'a BootConfig) -> Self {
        Self {
            boot_services,
            config,
            current_entry: config.default_entry.saturating_sub(1),
            framebuffer: None,
        }
    }

    /// Define framebuffer para renderização gráfica
    pub fn set_framebuffer(&mut self, fb: Framebuffer) {
        self.framebuffer = Some(fb);
    }

    /// Aguarda tecla de ativação do menu durante countdown
    ///
    /// Exibe countdown e aguarda usuário pressionar 'M' para abrir menu.
    /// Se timeout expirar, retorna false para boot automático.
    ///
    /// # Argumentos
    /// * `boot_services` - Serviços de boot UEFI
    /// * `timeout_seconds` - Tempo em segundos para countdown
    ///
    /// # Retorna
    /// `true` se usuário pressionou M, `false` se timeout expirou
    pub fn wait_for_trigger(boot_services: &BootServices, timeout_seconds: u64) -> bool {
        info!("");
        info!("═══════════════════════════════════════════════════");
        info!("  Pressione 'M' para abrir o menu de boot");
        info!("  Boot automático em {} segundos...", timeout_seconds);
        info!("═══════════════════════════════════════════════════");

        let stdin = boot_services.stdin();

        // Calcular ticks (verificar a cada 100ms)
        let tick_interval_ms = 100;
        let total_ticks = (timeout_seconds * 1000) / tick_interval_ms;

        for tick in 0..total_ticks {
            // Atualizar countdown a cada 1 segundo
            if tick % 10 == 0 {
                let remaining = timeout_seconds - (tick * tick_interval_ms) / 1000;
                if remaining > 0 {
                    info!("Boot automático em {} segundos... (M para menu)", remaining);
                }
            }

            // Verificar se há tecla pressionada
            match stdin.read_key() {
                Ok(Some(key)) => match key {
                    Key::Printable(c) if c.0 == 'M' as u16 || c.0 == 'm' as u16 => {
                        info!("Tecla M detectada! Abrindo menu interativo...");
                        return true;
                    },
                    _ => {},
                },
                _ => {},
            }

            // Aguardar 100ms (100,000 microsegundos)
            boot_services.stall(tick_interval_ms * 1000);
        }

        info!("Iniciando boot automático...");
        false
    }

    /// Exibi menu e aguarda seleção do usuário
    ///
    /// # Retorna
    /// Índice (0-based) da entrada selecionada
    pub fn show(&mut self) -> usize {
        info!("Exibindo menu interativo");

        // Se temos framebuffer, poderia renderizar graficamente
        // Por enquanto, usar modo texto sempre
        self.show_text()
    }

    /// Renderiza menu em modo texto usando ConOut UEFI
    fn show_text(&mut self) -> usize {
        let stdout = self.boot_services.stdout();

        loop {
            // Limpar tela
            let _ = stdout.reset(false);

            // Banner
            info!("");
            info!("═══════════════════════════════════════════════════");
            info!(
                "  {}",
                self.config
                    .interface_branding
                    .as_deref()
                    .unwrap_or("Ignite Bootloader")
            );
            info!("═══════════════════════════════════════════════════");
            info!("");

            // Lista de entradas
            info!("Selecione uma entrada de boot:");
            info!("");

            for (i, entry) in self.config.entries.iter().enumerate() {
                let marker = if i == self.current_entry { "►" } else { " " };

                if let Some(ref comment) = entry.comment {
                    info!("  {} [{}] {} - {}", marker, i + 1, entry.name, comment);
                } else {
                    info!("  {} [{}] {}", marker, i + 1, entry.name);
                }
            }

            info!("");
            info!("═══════════════════════════════════════════════════");
            info!("  ↑↓: Navegar  |  Enter: Boot  |  ESC: Cancelar");
            info!("  1-9: Seleção direta");
            info!("═══════════════════════════════════════════════════");

            // Processar input
            match self.wait_for_key() {
                MenuAction::Up => {
                    if self.current_entry > 0 {
                        self.current_entry -= 1;
                    } else {
                        // Wrap around para última entrada
                        self.current_entry = self.config.entries.len() - 1;
                    }
                },
                MenuAction::Down => {
                    if self.current_entry < self.config.entries.len() - 1 {
                        self.current_entry += 1;
                    } else {
                        // Wrap around para primeira entrada
                        self.current_entry = 0;
                    }
                },
                MenuAction::Select => {
                    let selected = &self.config.entries[self.current_entry];
                    info!("Entrada selecionada: {}", selected.name);
                    return self.current_entry;
                },
                MenuAction::Cancel => {
                    info!("Menu cancelado, usando entrada padrão");
                    return self
                        .config
                        .default_entry
                        .saturating_sub(1)
                        .min(self.config.entries.len() - 1);
                },
                MenuAction::Number(n) => {
                    let index = (n as usize).saturating_sub(1);
                    if index < self.config.entries.len() {
                        info!(
                            "Entrada {} selecionada: {}",
                            n, self.config.entries[index].name
                        );
                        return index;
                    }
                },
            }
        }
    }

    /// Aguarda input do usuário
    fn wait_for_key(&self) -> MenuAction {
        let stdin = self.boot_services.stdin();

        loop {
            match stdin.read_key() {
                Ok(Some(key)) => {
                    return match key {
                        Key::Special(scan_code) => match scan_code {
                            ScanCode::UP => MenuAction::Up,
                            ScanCode::DOWN => MenuAction::Down,
                            ScanCode::ESCAPE => MenuAction::Cancel,
                            _ => continue,
                        },
                        Key::Printable(c) => {
                            // Enter (CR ou LF)
                            if c.0 == '\r' as u16 || c.0 == '\n' as u16 {
                                return MenuAction::Select;
                            }

                            // Números 1-9 para seleção direta
                            if c.0 >= '1' as u16 && c.0 <= '9' as u16 {
                                let num = (c.0 - '0' as u16) as u8;
                                return MenuAction::Number(num);
                            }

                            continue;
                        },
                    };
                },
                _ => {
                    // Aguardar um pouco antes de tentar novamente (50ms)
                    self.boot_services.stall(50_000);
                },
            }
        }
    }

    /// Obter entrada atualmente selecionada
    pub fn current_entry(&self) -> &MenuEntry {
        &self.config.entries[self.current_entry]
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
}

/// Ações do menu
enum MenuAction {
    Up,
    Down,
    Select,
    Cancel,
    Number(u8),
}
