//! Módulo de segurança
//!
//! Responsável por verificação de integridade, proteção contra rollback e Secure Boot

pub mod integrity;
pub mod rollback;
pub mod secureboot;

pub use integrity::{IntegrityChecker, VerificationResult};
pub use rollback::{KernelVersion, RollbackProtection, RollbackResult};
pub use secureboot::{SecureBootManager, SecureBootState};
