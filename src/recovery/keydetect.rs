//! Detecção de teclas especiais durante boot
//!
//! Permite ao usuário pressionar teclas para entrar em modos especiais

use uefi::table::boot::BootServices;

/// Verifica se uma tecla especial foi pressionada
pub struct KeyDetector;

impl KeyDetector {
    /// Verifica se a tecla 'R' foi pressionada (modo recovery)
    ///
    /// TODO: Implementar detecção real de tecla via UEFI Input Protocol
    /// Por enquanto, sempre retorna false
    pub fn check_recovery_key(_boot_services: &BootServices) -> bool {
        // TODO: Implementar leitura de input UEFI
        // Exemplo de como seria:
        // 1. Obter protocolo SimpleTextInput
        // 2. Verificar se há tecla pressionada (non-blocking)
        // 3. Comparar com 'R' ou 'r'

        false // Por enquanto, sempre false
    }

    /// Exibe mensagem discreta sobre tecla de recovery
    pub fn show_recovery_hint() {
        // Mensagem no canto inferior direito (estilo Ctrl+Alt+Del)
        log::info!("Pressione 'R' para modo de recuperação");

        // TODO: Posicionar texto no canto da tela usando GOP
        // Por enquanto, apenas log
    }
}
