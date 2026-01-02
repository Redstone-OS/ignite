//! Carregador de Configuração
//!
//! Responsável por localizar e ler o arquivo de configuração do disco.

use super::{parser::Parser, types::BootConfig};
use crate::{
    core::error::Result,
    fs::{read_to_string, FileSystem},
};

const CONFIG_FILENAMES: &[&str] = &["EFI/BOOT/ignite.cfg", "boot/ignite.cfg"];

/// Tenta carregar a configuração. Retorna `BootConfig::recovery()` se falhar.
pub fn load_configuration(fs: &mut dyn FileSystem) -> Result<BootConfig> {
    let mut parser = Parser::new();

    // Tenta abrir a raiz do FS. Se falhar, é erro de I/O sério.
    let mut root = match fs.root() {
        Ok(r) => r,
        Err(_) => return Ok(BootConfig::recovery()),
    };

    for filename in CONFIG_FILENAMES {
        // Tenta abrir o arquivo
        if let Ok(mut file) = root.open_file(filename) {
            crate::println!("Carregando config: {}", filename);
            let content = match read_to_string(file.as_mut()) {
                Ok(c) => c,
                Err(_) => continue, // Arquivo ilegível, tenta próximo
            };

            // Se o parse falhar, retorna erro (não fallback silencioso)
            // para que o usuário saiba que o arquivo existe mas está errado.
            return parser.parse(&content);
        }
    }

    crate::println!("Nenhum arquivo de configuração encontrado.");
    // Se não encontrar, retorna configuração padrão (pode abrir um shell ou menu
    // default)
    Ok(BootConfig::default())
}
