//! Parser de Arquivos de Configuração
//!
//! Suporta um formato similar ao TOML/INI simplificado ou estilo Limine.
//!
//! Sintaxe:
//! chave: valor
//!
//! /Nome da Entrada
//!     protocol: linux
//!     path: boot():/vmlinuz

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use super::{
    macros::MacroExpander,
    types::{BootConfig, Entry, Module, Protocol},
};
use crate::core::error::{BootError, ConfigError, Result};

pub struct Parser {
    expander: MacroExpander,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            expander: MacroExpander::new(),
        }
    }

    pub fn parse(&mut self, content: &str) -> Result<BootConfig> {
        let mut config = BootConfig::default();
        let mut current_entry: Option<Entry> = None;

        let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

        for (line_num, line) in lines.iter().enumerate() {
            // Ignorar vazios e comentários
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Expansão de macros
            let line = self.expander.expand(line);

            // Detecção de nova entrada (começa com /)
            if let Some(name) = line.strip_prefix('/') {
                // Se tínhamos uma entrada sendo construída, salvamos ela
                if let Some(entry) = current_entry.take() {
                    config.entries.push(entry);
                }

                // Iniciar nova entrada
                current_entry = Some(Entry {
                    name:     name.trim().to_string(),
                    protocol: Protocol::Unknown,
                    path:     String::new(),
                    cmdline:  None,
                    modules:  Vec::new(),
                    dtb_path: None,
                });
                continue;
            }

            // Definição de macro (VAR = VAL)
            if let Some((key, val)) = line.split_once('=') {
                // Se a chave começa com $, é uma definição de macro interna
                let key = key.trim();
                if key.starts_with("${") && key.ends_with('}') {
                    let var_name = &key[2..key.len() - 1];
                    self.expander.set(var_name, val.trim());
                    continue;
                }
            }

            // Par Chave: Valor
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim();

                if let Some(entry) = &mut current_entry {
                    // Propriedades da Entrada
                    match key.as_str() {
                        "protocol" => entry.protocol = Protocol::from(val),
                        "path" | "kernel_path" => entry.path = val.to_string(),
                        "cmdline" | "kernel_cmdline" => entry.cmdline = Some(val.to_string()),
                        "module_path" => entry.modules.push(Module {
                            path:    val.to_string(),
                            cmdline: None,
                        }),
                        "dtb_path" => entry.dtb_path = Some(val.to_string()),
                        _ => {}, // Ignorar desconhecido
                    }
                } else {
                    // Propriedades Globais
                    match key.as_str() {
                        "timeout" => config.timeout = val.parse().ok(),
                        "default_entry" => {
                            // Tenta parsear como número (1-based index)
                            if let Ok(idx) = val.parse::<usize>() {
                                if idx > 0 {
                                    config.default_entry_idx = idx - 1;
                                }
                            }
                        },
                        "serial" => {
                            config.serial_enabled = val.eq_ignore_ascii_case("yes") || val == "true"
                        },
                        "quiet" => config.quiet = val.eq_ignore_ascii_case("yes") || val == "true",
                        "wallpaper" => config.wallpaper = Some(val.to_string()),
                        _ => {},
                    }
                }
                continue;
            }
        }

        // Adicionar última entrada pendente
        if let Some(entry) = current_entry {
            config.entries.push(entry);
        }

        self.validate(&config)?;
        Ok(config)
    }

    fn validate(&self, config: &BootConfig) -> Result<()> {
        if config.entries.is_empty() {
            // Não é necessariamente um erro fatal, mas avisa
            // log::warn!("Nenhuma entrada de boot encontrada na
            // configuração.");
        }
        Ok(())
    }
}
