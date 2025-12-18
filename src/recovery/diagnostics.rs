//! Diagnóstico de sistema
//!
//! Verifica integridade de arquivos e detecta problemas comuns
//! NOTA: Diagnósticos são informativos e não bloqueiam o boot

use crate::error::Result;
use crate::fs::FileLoader;
use log::{info, warn};

/// Sistema de diagnóstico
pub struct Diagnostics;

impl Diagnostics {
    /// Executa diagnóstico básico do sistema
    /// 
    /// Verifica arquivos essenciais mas NÃO bloqueia o boot em caso de erro.
    /// Apenas registra warnings para revisão futura.
    pub fn run_basic_diagnostics(file_loader: &mut FileLoader) {
        info!("Executando diagnóstico básico...");

        // Verificar kernel principal
        Self::check_file(file_loader, "forge", true);

        // Verificar InitFS (opcional)
        Self::check_file(file_loader, "initfs", false);

        // TODO: Adicionar verificação de integridade com checksums
        // TODO: Adicionar verificação de espaço em disco
        // TODO: Adicionar verificação de firmware UEFI

        info!("Diagnóstico concluído.");
    }

    /// Verifica se um arquivo existe
    fn check_file(file_loader: &mut FileLoader, filename: &'static str, required: bool) {
        match file_loader.try_load_file(filename) {
            Ok(Some(file)) => {
                info!("✓ {} encontrado ({} bytes)", filename, file.size);
            }
            Ok(None) => {
                if required {
                    warn!("✗ {} NÃO encontrado (obrigatório)", filename);
                } else {
                    info!("○ {} não encontrado (opcional)", filename);
                }
            }
            Err(e) => {
                warn!("✗ Erro ao verificar {}: {:?}", filename, e);
            }
        }
    }

    /// Verifica integridade de um arquivo
    /// 
    /// TODO: Implementar verificação real com checksums/hashes
    pub fn verify_integrity(_filename: &'static str) -> bool {
        // TODO: Calcular hash do arquivo e comparar com valor esperado
        // Por enquanto, sempre retorna true
        true
    }
}
