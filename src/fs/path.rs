//! Utilitários de Manipulação de Caminhos
//!
//! Normaliza caminhos entre estilos Unix (usado na config) e UEFI (usado no
//! firmware).

use alloc::string::{String, ToString};

/// Normaliza um caminho para o formato UEFI (separador `\`).
/// Remove prefixos como `boot:` ou `/`.
pub fn normalize_path(path: &str) -> String {
    let mut p = path.replace('/', "\\");

    // Remove prefixos de volume comuns em configs
    if p.starts_with("boot:") {
        p = p.replace("boot:", "");
    }

    // Remove barra inicial (UEFI muitas vezes prefere caminhos relativos à raiz do
    // volume)
    if p.starts_with('\\') {
        p.remove(0);
    }

    p
}

/// Separa o nome do arquivo do diretório pai.
pub fn split_filename(path: &str) -> (String, String) {
    let normalized = normalize_path(path);
    if let Some(idx) = normalized.rfind('\\') {
        let dir = normalized[..idx].to_string();
        let file = normalized[idx + 1..].to_string();
        (dir, file)
    } else {
        (String::new(), normalized)
    }
}
