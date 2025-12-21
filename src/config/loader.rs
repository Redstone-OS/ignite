//! Carregador de Configuração
//!
//! Responsável por localizar e ler o arquivo de configuração do disco.

use super::{parser::Parser, types::BootConfig};
use crate::{
    core::error::{BootError, ConfigError, Result},
    fs::{FileSystem, read_to_string},
};

/// Nomes de arquivo padrão para procurar.
const CONFIG_FILENAMES: &[&str] = &[
    "ignite.cfg",
    "boot/ignite.cfg",
    "limine.cfg", // Compatibilidade
    "boot/limine.cfg",
];

/// Tenta carregar a configuração de um sistema de arquivos.
pub fn load_configuration(fs: &mut dyn FileSystem) -> Result<BootConfig> {
    let mut parser = Parser::new();
    let root = fs.root()?;

    for filename in CONFIG_FILENAMES {
        // Tenta abrir o arquivo
        if let Ok(mut file) = root.open_file(filename) {
            crate::println!("Carregando configuração de: {}", filename);

            // Lê todo o conteúdo
            let content = read_to_string(file.as_mut())?;

            // Parseia
            return parser.parse(&content);
        }
    }

    crate::println!("Nenhum arquivo de configuração encontrado. Usando padrões.");
    // Se não encontrar, retorna configuração padrão (pode abrir um shell ou menu
    // default)
    Ok(BootConfig::default())
}
