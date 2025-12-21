//! Gerenciador de Recuperação
//!
//! Decide qual kernel carregar baseado no histórico de falhas e input do
//! usuário.

use super::state::PersistentState;
use crate::{
    config::{BootConfig, Entry},
    ui::input::{InputManager, Key},
};

/// Limite de falhas antes de forçar o modo de recuperação.
const MAX_FAILURES: u8 = 3;

pub struct RecoveryManager {
    state: PersistentState,
}

impl RecoveryManager {
    pub fn new() -> Self {
        Self {
            state: PersistentState::load(),
        }
    }

    /// Verifica se o usuário está segurando a tecla de recuperação (R ou
    /// Shift).
    fn check_force_keys(&self) -> bool {
        let input = InputManager::new();
        // Verifica se há tecla pressionada sem bloquear (poll)
        if let Some(key) = input.poll() {
            match key {
                Key::Char('r') | Key::Char('R') => return true,
                _ => {},
            }
        }
        false
    }

    /// Seleciona a entrada de boot apropriada.
    ///
    /// # Lógica
    /// 1. Se tecla 'R' pressionada -> Recovery.
    /// 2. Se falhas consecutivas > 3 -> Recovery.
    /// 3. Caso contrário -> Entrada Padrão (Config).
    pub fn select_entry<'a>(&mut self, config: &'a BootConfig) -> &'a Entry {
        let force_recovery = self.check_force_keys();
        let too_many_failures = self.state.failed_attempts >= MAX_FAILURES;

        if force_recovery || too_many_failures {
            if force_recovery {
                crate::println!("Recuperação: Solicitada pelo usuário (Tecla R).");
            } else {
                crate::println!(
                    "Recuperação: Detectadas {} falhas consecutivas.",
                    self.state.failed_attempts
                );
            }

            // Tenta encontrar uma entrada marcada como 'fallback' ou 'recovery' no nome
            // Ou a última entrada da lista (convenção comum)
            if let Some(recovery) = self.find_recovery_entry(config) {
                crate::println!("Usando entrada de recuperação: {}", recovery.name);
                return recovery;
            }

            crate::println!("AVISO: Nenhuma entrada de recuperação encontrada. Tentando padrão.");
        }

        // Caminho feliz
        let idx = config
            .default_entry_idx
            .min(config.entries.len().saturating_sub(1));

        // Registra que estamos tentando esta entrada
        self.state.mark_attempt(idx);

        &config.entries[idx]
    }

    fn find_recovery_entry<'a>(&self, config: &'a BootConfig) -> Option<&'a Entry> {
        // 1. Procurar por nome explícito
        for entry in &config.entries {
            let name = entry.name.to_lowercase();
            if name.contains("recovery") || name.contains("rescue") || name.contains("fallback") {
                return Some(entry);
            }
        }

        // 2. Fallback: última entrada (assumindo que kernels antigos/estáveis ficam no
        //    fim)
        config.entries.last()
    }
}
