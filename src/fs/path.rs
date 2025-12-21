//! Utilitários de Manipulação de Caminhos
//!
//! Normaliza caminhos entre estilos Unix (usado na config) e UEFI (usado no
//! firmware).

use alloc::string::{String, ToString};

/// Normaliza um caminho para o formato UEFI (separador `\`).
/// Remove prefixos como `boot():`, `boot:` ou `/` inicial.
pub fn normalize_path(path: &str) -> String {
    // 1. Unificar separadores para o padrão UEFI (\)
    let mut p = path.replace('/', "\\");

    // 2. Remover prefixos de dispositivo conhecidos
    // A ordem importa: strings mais longas primeiro
    if p.starts_with("boot():") {
        p = p[7..].to_string(); // Remove "boot():"
    } else if p.starts_with("boot:") {
        p = p[5..].to_string(); // Remove "boot:"
    } else if p.starts_with("vol():") {
        p = p[6..].to_string();
    }

    // 3. Remover barra invertida inicial
    // UEFI SimpleFileSystem espera caminhos relativos à raiz, ex: "EFI\BOOT\..."
    // e não "\EFI\BOOT\..."
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
