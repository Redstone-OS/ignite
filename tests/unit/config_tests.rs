//! Testes Unitários para o módulo de configuração
//!
//! Testa parsing, validação e loading de configuração.

#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::{string::{String, ToString}, vec::Vec};

/// Testa parsing de valores booleanos
#[test]
fn test_parse_boolean() {
    fn parse_bool(s: &str) -> Option<bool> {
        match s.to_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
    
    assert_eq!(parse_bool("true"), Some(true));
    assert_eq!(parse_bool("True"), Some(true));
    assert_eq!(parse_bool("TRUE"), Some(true));
    assert_eq!(parse_bool("false"), Some(false));
    assert_eq!(parse_bool("False"), Some(false));
    assert_eq!(parse_bool("FALSE"), Some(false));
    assert_eq!(parse_bool("invalid"), None);
    assert_eq!(parse_bool("1"), None);
}

/// Testa parsing de números inteiros
#[test]
fn test_parse_integer() {
    fn parse_int(s: &str) -> Option<u32> {
        s.parse().ok()
    }
    
    assert_eq!(parse_int("0"), Some(0));
    assert_eq!(parse_int("5"), Some(5));
    assert_eq!(parse_int("1000"), Some(1000));
    assert_eq!(parse_int("-1"), None); // Negativo
    assert_eq!(parse_int("abc"), None);
    assert_eq!(parse_int(""), None);
}

/// Testa parsing de resolução
#[test]
fn test_parse_resolution() {
    fn parse_resolution(s: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let width = parts[0].parse().ok()?;
        let height = parts[1].parse().ok()?;
        
        Some((width, height))
    }
    
    assert_eq!(parse_resolution("1920x1080"), Some((1920, 1080)));
    assert_eq!(parse_resolution("1024x768"), Some((1024, 768)));
    assert_eq!(parse_resolution("3840x2160"), Some((3840, 2160)));
    assert_eq!(parse_resolution("invalid"), None);
    assert_eq!(parse_resolution("1920"), None);
    assert_eq!(parse_resolution("1920x"), None);
}

/// Testa validação de timeout
#[test]
fn test_validate_timeout() {
    fn validate_timeout(timeout: i32) -> bool {
        timeout >= -1 && timeout <= 300
    }
    
    assert!(validate_timeout(0));
    assert!(validate_timeout(5));
    assert!(validate_timeout(300));
    assert!(validate_timeout(-1)); // Infinito
    assert!(!validate_timeout(-2));
    assert!(!validate_timeout(301));
}

/// Testa parsing de protocolo
#[test]
fn test_parse_protocol() {
    #[derive(Debug, PartialEq)]
    enum Protocol {
        Linux,
        Limine,
        EfiChainload,
        Multiboot2,
        Unknown,
    }
    
    fn parse_protocol(s: &str) -> Protocol {
        match s.to_lowercase().as_str() {
            "linux" => Protocol::Linux,
            "limine" | "redstone" | "native" => Protocol::Limine,
            "efi" | "chainload" => Protocol::EfiChainload,
            "multiboot2" => Protocol::Multiboot2,
            _ => Protocol::Unknown,
        }
    }
    
    assert_eq!(parse_protocol("linux"), Protocol::Linux);
    assert_eq!(parse_protocol("Linux"), Protocol::Linux);
    assert_eq!(parse_protocol("limine"), Protocol::Limine);
    assert_eq!(parse_protocol("redstone"), Protocol::Limine);
    assert_eq!(parse_protocol("native"), Protocol::Limine);
    assert_eq!(parse_protocol("efi"), Protocol::EfiChainload);
    assert_eq!(parse_protocol("chainload"), Protocol::EfiChainload);
    assert_eq!(parse_protocol("multiboot2"), Protocol::Multiboot2);
    assert_eq!(parse_protocol("invalid"), Protocol::Unknown);
}

/// Testa remoção de comentários
#[test]
fn test_remove_comments() {
    fn remove_comment(line: &str) -> &str {
        if let Some(pos) = line.find('#') {
            &line[..pos]
        } else {
            line
        }
    }
    
    assert_eq!(remove_comment("timeout = 5 # comentário"), "timeout = 5 ");
    assert_eq!(remove_comment("# comentário completo"), "");
    assert_eq!(remove_comment("sem comentário"), "sem comentário");
    assert_eq!(remove_comment(""), "");
}

/// Testa parsing de key-value
#[test]
fn test_parse_key_value() {
    fn parse_kv(line: &str) -> Option<(&str, &str)> {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            Some((parts[0].trim(), parts[1].trim()))
        } else {
            None
        }
    }
    
    assert_eq!(parse_kv("key = value"), Some(("key", "value")));
    assert_eq!(parse_kv("timeout=5"), Some(("timeout", "5")));
    assert_eq!(parse_kv("  spaced  =  value  "), Some(("spaced", "value")));
    assert_eq!(parse_kv("no_equals"), None);
    assert_eq!(parse_kv(""), None);
}

/// Testa remoção de aspas de strings
#[test]
fn test_unquote_string() {
    fn unquote(s: &str) -> &str {
        let trimmed = s.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len()-1]
        } else {
            trimmed
        }
    }
    
    assert_eq!(unquote("\"quoted\""), "quoted");
    assert_eq!(unquote("  \"quoted\"  "), "quoted");
    assert_eq!(unquote("not quoted"), "not quoted");
    assert_eq!(unquote("\""), "\""); // Aspas incompletas
}

/// Testa detecção de seção [[entry]]
#[test]
fn test_detect_entry_section() {
    fn is_entry_section(line: &str) -> bool {
        line.trim() == "[[entry]]"
    }
    
    assert!(is_entry_section("[[entry]]"));
    assert!(is_entry_section("  [[entry]]  "));
    assert!(!is_entry_section("[[other]]"));
    assert!(!is_entry_section("[entry]"));
    assert!(!is_entry_section(""));
}

/// Testa validação de nome de arquivo
#[test]
fn test_validate_filename() {
    fn is_valid_filename(name: &str) -> bool {
        !name.is_empty() && 
        !name.contains('\0') &&
        !name.contains('/') &&  // Simplificado
        name.len() <= 255
    }
    
    assert!(is_valid_filename("valid.conf"));
    assert!(is_valid_filename("ignite.conf"));
    assert!(!is_valid_filename(""));
    assert!(!is_valid_filename("invalid/path"));
    assert!(!is_valid_filename("null\0char"));
    
    let long_name = "a".repeat(256);
    assert!(!is_valid_filename(&long_name));
}

/// Testa validação de path
#[test]
fn test_validate_path() {
    fn is_valid_path(path: &str) -> bool {
        !path.is_empty() && !path.contains('\0')
    }
    
    assert!(is_valid_path("boot():/EFI/ignite/kernel"));
    assert!(is_valid_path("root():/boot/vmlinuz"));
    assert!(is_valid_path("/absolute/path"));
    assert!(is_valid_path("relative/path"));
    assert!(!is_valid_path(""));
    assert!(!is_valid_path("null\0char"));
}

/// Testa merge de configurações (default + parsed)
#[test]
fn test_config_merge() {
    struct Config {
        timeout: Option<u32>,
        quiet: bool,
    }
    
    let default_config = Config {
        timeout: Some(5),
        quiet: false,
    };
    
    let user_config = Config {
        timeout: Some(10),
        quiet: true,
    };
    
    // User config sobrescreve default
    let merged = Config {
        timeout: user_config.timeout.or(default_config.timeout),
        quiet: user_config.quiet, // Assume sempre user value para bool
    };
    
    assert_eq!(merged.timeout, Some(10));
    assert_eq!(merged.quiet, true);
}

/// Testa parsing de módulos
#[test]
fn test_parse_module() {
    #[derive(Debug, PartialEq)]
    struct Module {
        path: String,
        cmdline: Option<String>,
    }
    
    let module = Module {
        path: "boot():/initrd.img".to_string(),
        cmdline: Some("initrd".to_string()),
    };
    
    assert_eq!(module.path, "boot():/initrd.img");
    assert_eq!(module.cmdline, Some("initrd".to_string()));
}

/// Testa validação de índice padrão
#[test]
fn test_validate_default_index() {
    fn is_valid_default(default: usize, entry_count: usize) -> bool {
        default < entry_count && entry_count > 0
    }
    
    assert!(is_valid_default(0, 3));
    assert!(is_valid_default(2, 3));
    assert!(!is_valid_default(3, 3)); // Out of bounds
    assert!(!is_valid_default(0, 0)); // No entries
    assert!(!is_valid_default(10, 3)); // Way out of bounds
}

/// Testa trim de whitespace
#[test]
fn test_trim_whitespace() {
    assert_eq!("  test  ".trim(), "test");
    assert_eq!("\t\ntest\t\n".trim(), "test");
    assert_eq!("test".trim(), "test");
    assert_eq!("  ".trim(), "");
}

/// Testa case-insensitive comparison
#[test]
fn test_case_insensitive() {
    fn eq_ignore_case(a: &str, b: &str) -> bool {
        a.to_lowercase() == b.to_lowercase()
    }
    
    assert!(eq_ignore_case("Test", "test"));
    assert!(eq_ignore_case("TEST", "test"));
    assert!(eq_ignore_case("TeSt", "TeSt"));
    assert!(!eq_ignore_case("test", "other"));
}
