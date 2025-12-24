//! Códigos de Cores ANSI para Terminal Serial
//!
//! Fornece constantes para colorir a saída na serial usando escape codes ANSI.

/// Reset para cor padrão
pub const RESET: &str = "\x1b[0m";

/// Cores de texto
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";
pub const GRAY: &str = "\x1b[90m";

/// Cores bright/bold
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_BLUE: &str = "\x1b[94m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";

/// Bold
pub const BOLD: &str = "\x1b[1m";

/// Macro para colorir tags [OK], [INFO], [DEBUG], [ERROR]
#[macro_export]
macro_rules! colorize_tag {
    ($text:expr) => {{
        let text = $text;
        if text.contains("[OK]") {
            text.replace(
                "[OK]",
                &alloc::format!("{}{BOLD}[OK]{RESET}", $crate::core::colors::BRIGHT_GREEN),
            )
        } else if text.contains("[INFO]") {
            text.replace(
                "[INFO]",
                &alloc::format!("{}{BOLD}[INFO]{RESET}", $crate::core::colors::BRIGHT_BLUE),
            )
        } else if text.contains("[DEBUG]") {
            text.replace(
                "[DEBUG]",
                &alloc::format!(
                    "{}{BOLD}[DEBUG]{RESET}",
                    $crate::core::colors::BRIGHT_MAGENTA
                ),
            )
        } else if text.contains("[ERROR]") || text.contains("[ERRO]") {
            text.replace(
                "[ERROR]",
                &alloc::format!("{}{BOLD}[ERROR]{RESET}", $crate::core::colors::BRIGHT_RED),
            )
            .replace(
                "[ERRO]",
                &alloc::format!("{}{BOLD}[ERRO]{RESET}", $crate::core::colors::BRIGHT_RED),
            )
        } else if text.contains("[WARN]") || text.contains("[AVISO]") {
            text.replace(
                "[WARN]",
                &alloc::format!("{}{BOLD}[WARN]{RESET}", $crate::core::colors::BRIGHT_YELLOW),
            )
            .replace(
                "[AVISO]",
                &alloc::format!(
                    "{}{BOLD}[AVISO]{RESET}",
                    $crate::core::colors::BRIGHT_YELLOW
                ),
            )
        } else {
            text.to_string()
        }
    }};
}

/// Função auxiliar para colorir automaticamente
pub fn colorize(text: &str) -> alloc::string::String {
    use alloc::string::ToString;

    let mut result = text.to_string();

    // [OK] -> Verde
    if result.contains("[OK]") {
        result = result.replace("[OK]", &alloc::format!("{}{}[OK]{}", BRIGHT_GREEN, BOLD, RESET));
    }

    // [INFO] -> Azul
    if result.contains("[INFO]") {
        result = result.replace("[INFO]", &alloc::format!("{}{}[INFO]{}", BRIGHT_BLUE, BOLD, RESET));
    }

    // [DEBUG] -> Magenta/Rosa
    if result.contains("[DEBUG]") {
        result = result.replace(
            "[DEBUG]",
            &alloc::format!("{}{}[DEBUG]{}", BRIGHT_MAGENTA, BOLD, RESET),
        );
    }

    // [ERROR]/[ERRO] -> Vermelho
    if result.contains("[ERROR]") {
        result = result.replace(
            "[ERROR]",
            &alloc::format!("{}{}[ERROR]{}", BRIGHT_RED, BOLD, RESET),
        );
    }
    if result.contains("[ERRO]") {
        result = result.replace("[ERRO]", &alloc::format!("{}{}[ERRO]{}", BRIGHT_RED, BOLD, RESET));
    }

    // [WARN]/[AVISO] -> Amarelo
    if result.contains("[WARN]") {
        result = result.replace(
            "[WARN]",
            &alloc::format!("{}{}[WARN]{}", BRIGHT_YELLOW, BOLD, RESET),
        );
    }
    if result.contains("[AVISO]") {
        result = result.replace(
            "[AVISO]",
            &alloc::format!("{}{}[AVISO]{}", BRIGHT_YELLOW, BOLD, RESET),
        );
    }

    result
}

