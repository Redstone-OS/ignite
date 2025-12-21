//! Carregador de Configuração
//!
//! Responsável por localizar e ler o arquivo de configuração do disco.

use super::{parser::Parser, types::BootConfig};
use crate::{
    core::error::{BootError, ConfigError, Result},
    fs::{FileSystem, read_to_string},
};

const CONFIG_FILENAMES: &[&str] = &["ignite.cfg", "boot/ignite.cfg"];

pub fn load_configuration(fs: &mut dyn FileSystem) -> Result<BootConfig> {
    let mut parser = Parser::new();
    // FIX: Mutabilidade
    let mut root = fs.root()?;

    for filename in CONFIG_FILENAMES {
        // Tenta abrir o arquivo
        if let Ok(mut file) = root.open_file(filename) {
            crate::println!("Carregando config: {}", filename);
            let content = read_to_string(file.as_mut())?;
            return parser.parse(&content);
        }
    }

    crate::println!("Nenhum arquivo de configuração encontrado. Usando padrões.");
    // Se não encontrar, retorna configuração padrão (pode abrir um shell ou menu
    // default)
    Ok(BootConfig::default())
}
