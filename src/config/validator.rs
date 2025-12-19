//! Configuration Validator
//!
//! Validates configuration syntax and values

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::types::BootConfig;
use crate::error::{BootError, Result};

/// Validation error
#[derive(Debug)]
pub struct ValidationError {
    pub line:    usize,
    pub message: String,
}

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate parsed configuration
    pub fn validate(config: &BootConfig) -> Result<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check if there are any entries
        if config.entries.is_empty() {
            errors.push(ValidationError {
                line:    0,
                message: "No boot entries defined".to_string(),
            });
        }

        // Validate default entry
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

        // Validate each entry
        for (i, entry) in config.entries.iter().enumerate() {
            // Check required fields
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

            // Validate protocol name
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
