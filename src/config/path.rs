//! Sintaxe de Caminhos do Ignite
//!
//! Interpreta strings como `boot(1):/kernel` ou
//! `guid(xxx):/efi/boot/bootx64.efi`.

use alloc::string::{String, ToString};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceSpecifier {
    /// `boot()`: Dispositivo onde o bootloader está rodando.
    BootVolume,
    /// `boot(N)`: Partição N do disco de boot.
    BootPartition(u32),
    /// `uuid(X)`: Partição com UUID X.
    Uuid(String),
    /// `label(X)`: Partição com Label X.
    Label(String),
}

/// Representa um caminho completo parseado da config.
#[derive(Debug, Clone)]
pub struct ConfigPath {
    pub device: DeviceSpecifier,
    pub path:   String,
}

impl ConfigPath {
    pub fn parse(input: &str) -> Option<Self> {
        // Verifica se tem separador de dispositivo (:)
        if let Some((dev_part, path_part)) = input.split_once(':') {
            // Parse do dispositivo: nome(arg)
            let dev_spec = if dev_part == "boot()" {
                DeviceSpecifier::BootVolume
            } else if dev_part.starts_with("boot(") && dev_part.ends_with(')') {
                let num_str = &dev_part[5..dev_part.len() - 1];
                let num = num_str.parse::<u32>().ok()?;
                DeviceSpecifier::BootPartition(num)
            } else if dev_part.starts_with("uuid(") && dev_part.ends_with(')') {
                let uuid = &dev_part[5..dev_part.len() - 1];
                DeviceSpecifier::Uuid(uuid.to_string())
            } else {
                // Default ou desconhecido, tratar como caminho relativo se não tiver parenteses
                return None;
            };

            Some(Self {
                device: dev_spec,
                path:   path_part.to_string(),
            })
        } else {
            // Caminho sem dispositivo explícito -> Assume BootVolume
            Some(Self {
                device: DeviceSpecifier::BootVolume,
                path:   input.to_string(),
            })
        }
    }
}
