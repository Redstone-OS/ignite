//! Macro System
//!
//! Supports ${MACRO_NAME} expansion in configuration

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
};

/// Macro storage and expansion
pub struct MacroExpander {
    macros: BTreeMap<String, String>,
}

impl MacroExpander {
    pub fn new() -> Self {
        let mut expander = Self {
            macros: BTreeMap::new(),
        };

        // Built-in macros
        expander.define_builtin();

        expander
    }

    /// Define built-in macros
    fn define_builtin(&mut self) {
        // Architecture
        #[cfg(target_arch = "x86_64")]
        self.macros.insert("ARCH".to_string(), "x86-64".to_string());

        #[cfg(target_arch = "x86")]
        self.macros.insert("ARCH".to_string(), "ia-32".to_string());

        #[cfg(target_arch = "aarch64")]
        self.macros
            .insert("ARCH".to_string(), "aarch64".to_string());

        #[cfg(target_arch = "riscv64")]
        self.macros
            .insert("ARCH".to_string(), "riscv64".to_string());

        // Firmware type
        #[cfg(target_os = "uefi")]
        self.macros
            .insert("FW_TYPE".to_string(), "UEFI".to_string());

        // TODO: Add BIOS when supported
        // #[cfg(target_os = "bios")]
        // self.macros.insert("FW_TYPE".to_string(), "BIOS".to_string());
    }

    /// Define a custom macro
    pub fn define(&mut self, name: String, value: String) {
        self.macros.insert(name, value);
    }

    /// Expand macros in a string
    pub fn expand(&self, input: &str) -> String {
        let mut result = String::from(input);

        // Find and replace all ${MACRO} occurrences
        for (name, value) in &self.macros {
            let pattern = format!("${{{}}}", name);
            result = result.replace(&pattern, value);
        }

        result
    }

    /// Check if a macro is defined
    pub fn is_defined(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    /// Get macro value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.macros.get(name).map(|s| s.as_str())
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_macros() {
        let expander = MacroExpander::new();
        assert!(expander.is_defined("ARCH"));
        assert!(expander.is_defined("FW_TYPE"));
    }

    #[test]
    fn test_expansion() {
        let mut expander = MacroExpander::new();
        expander.define("TEST".to_string(), "value".to_string());

        let result = expander.expand("Hello ${TEST} world");
        assert_eq!(result, "Hello value world");
    }

    #[test]
    fn test_multiple_expansion() {
        let mut expander = MacroExpander::new();
        expander.define("A".to_string(), "foo".to_string());
        expander.define("B".to_string(), "bar".to_string());

        let result = expander.expand("${A} and ${B}");
        assert_eq!(result, "foo and bar");
    }
}
