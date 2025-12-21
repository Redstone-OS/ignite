//! Subsistema de Recuperação e Alta Disponibilidade
//!
//! Garante que o sistema possa inicializar mesmo em caso de corrupção ou falhas
//! repetidas.
//!
//! Funcionalidades:
//! - **A/B Boot:** Detecção de falhas e fallback automático.
//! - **Persistência:** Contagem de tentativas na NVRAM.
//! - **Diagnóstico:** Verificação pré-boot de arquivos.

pub mod diagnostics;
pub mod manager;
pub mod state;

// Re-exports
pub use diagnostics::Diagnostics;
pub use manager::RecoveryManager;
