//! Políticas de Segurança
//!
//! Define o comportamento do bootloader quando verificações de integridade
//! falham.

use crate::config::BootConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyAction {
    /// Bloqueia o boot imediatamente (Modo Seguro).
    Halt,
    /// Loga o erro e continua (Modo Permissivo/Desenvolvimento).
    WarnAndContinue,
    /// Tenta um fallback (ex: kernel anterior).
    Fallback,
}

pub struct SecurityPolicy {
    secure_boot:    bool,
    developer_mode: bool,
}

impl SecurityPolicy {
    /// Carrega a política baseada na configuração e no estado do hardware.
    pub fn new(config: &BootConfig) -> Self {
        let sb_active = super::secure_boot::enforcement_required();

        Self {
            secure_boot:    sb_active,
            // FIX: Usar !quiet no lugar de verbose (já que verbose não existe)
            developer_mode: !sb_active && !config.quiet,
        }
    }

    /// Decide o que fazer em caso de falha de verificação de assinatura.
    pub fn on_signature_fail(&self) -> PolicyAction {
        if self.secure_boot {
            crate::println!("CRÍTICO: Violação de Secure Boot.");
            PolicyAction::Halt
        } else {
            crate::println!("AVISO: Assinatura inválida (SB inativo).");
            PolicyAction::WarnAndContinue
        }
    }

    /// Decide o que fazer em caso de falha de integridade (Hash mismatch).
    pub fn on_integrity_fail(&self) -> PolicyAction {
        if self.developer_mode {
            PolicyAction::WarnAndContinue
        } else {
            crate::println!("ERRO: Arquivo corrompido detectado.");
            PolicyAction::Halt
        }
    }
}
