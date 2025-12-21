//! Validador de Configuração
//!
//! Valida sintaxe e valores de configuração

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::types::BootConfig;
use crate::core::error::Result;

/// Erro de validação
#[derive(Debug)]
pub struct ValidationError {
    pub line:    usize,
    pub message: String,
}

/// Validador de configuração
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validar configuração parseada
    pub fn validate(config: &BootConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Verificar se existem entradas
        if config.entries.is_empty() {
            errors.push(ValidationError {
                line:    0,
                message: "No boot entries defined".to_string(),
            });
        }

        // Validar entrada padrão
        if config.default_entry == 0 || config.default_entry > config.entries.len() {
            errors.push(ValidationError {
                line:    0,
                message: format!(
                    "Invalid default_entry: {} (must be 1-{})",
                    config.default_entry,
                    config.entries.len()
                ),
            });
        }

        // Validar cada entrada
        for (i, entry) in config.entries.iter().enumerate() {
            // Verificar campos obrigatórios
            if entry.protocol.is_empty() {
                errors.push(ValidationError {
                    line:    i + 1,
                    message: format!("Entry '{}': missing protocol", entry.name),
                });
            }

            if entry.kernel_path.is_empty() {
                errors.push(ValidationError {
                    line:    i + 1,
                    message: format!("Entry '{}': missing kernel_path", entry.name),
                });
            }

            // Validar nome do protocolo
            let valid_protocols = [
                "limine",
                "linux",
                "multiboot",
                "multiboot1",
                "multiboot2",
                "efi",
                "bios",
            ];
            if !valid_protocols.contains(&entry.protocol.to_lowercase().as_str()) {
                errors.push(ValidationError {
                    line:    i + 1,
                    message: format!(
                        "Entry '{}': unknown protocol '{}'",
                        entry.name, entry.protocol
                    ),
                });
            }
        }

        Ok(errors)
    }
}
