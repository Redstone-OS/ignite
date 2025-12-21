//! Motor de Expansão de Macros
//!
//! Permite substituição de variáveis na configuração (ex: `${ARCH}`).

use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};

pub struct MacroExpander {
    variables: BTreeMap<String, String>,
}

impl MacroExpander {
    pub fn new() -> Self {
        let mut me = Self {
            variables: BTreeMap::new(),
        };
        me.populate_defaults();
        me
    }

    fn populate_defaults(&mut self) {
        // Arquitetura atual
        #[cfg(target_arch = "x86_64")]
        self.set("ARCH", "x86_64");
        #[cfg(target_arch = "aarch64")]
        self.set("ARCH", "aarch64");

        self.set("VERSION", env!("CARGO_PKG_VERSION"));
        self.set("BOOTLOADER", "Ignite");
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Expande todas as ocorrências de `${VAR}` na string de entrada.
    pub fn expand(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Loop simples para substituir.
        // Em um sistema mais complexo, faríamos parsing de tokens.
        for (key, val) in &self.variables {
            let pattern = alloc::format!("${{{}}}", key); // ${KEY}
            result = result.replace(&pattern, val);
        }

        result
    }
}
