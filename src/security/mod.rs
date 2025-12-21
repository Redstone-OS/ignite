//! Subsistema de Segurança
//!
//! Responsável pela integridade da cadeia de boot:
//! - Detecção de Secure Boot
//! - Medição TPM (Trusted Boot)
//! - Políticas de execução

pub mod policy;
pub mod secure_boot;
pub mod tpm;
// pub mod verify; // Futuro: Verificação PE/COFF manual se necessário

// Re-exports
pub use policy::{PolicyAction, SecurityPolicy};
pub use secure_boot::{SecureBootState, get_state};
pub use tpm::measure_binary;

/// Função helper para validar e medir um arquivo carregado.
pub fn validate_and_measure(
    data: &[u8],
    name: &str,
    policy: &SecurityPolicy,
) -> crate::core::error::Result<()> {
    // 1. Medir no TPM (se disponível)
    // PCR 9 é comumente usado para o Kernel/Bootloader payload
    tpm::measure_binary(data, 9, name)?;

    // 2. Verificar Secure Boot (Se aplicável)
    // Nota: Se carregado via LoadImage() do UEFI, o firmware já verificou.
    // Se carregado manualmente (ELF), precisaríamos verificar a assinatura aqui.
    if secure_boot::enforcement_required() {
        // TODO: Verificar assinatura Authenticode ou GPG interna
        // Se falhar:
        // match policy.on_signature_fail() { ... }
    }

    Ok(())
}
