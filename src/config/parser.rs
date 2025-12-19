//! Configuration Parser
//!
//! Parses boot configuration files in Limine-compatible format

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use log::info;

use super::{
    macros::MacroExpander,
    types::{BootConfig, MenuEntry, Module, WallpaperStyle},
};
use crate::error::{BootError, Result};

/// Configuration file parser
pub struct ConfigParser {
    lines:        Vec<String>,
    macros:       MacroExpander,
    current_line: usize,
}

impl ConfigParser {
    /// Parse configuration from string content
    pub fn parse(content: &str) -> Result<BootConfig> {
        let mut parser = Self {
            lines:        content.lines().map(|s| s.trim().to_string()).collect(),
            macros:       MacroExpander::new(),
            current_line: 0,
        };

        parser.parse_config()
    }

    fn parse_config(&mut self) -> Result<BootConfig> {
        let mut config = BootConfig::default();

        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line].clone();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                self.current_line += 1;
                continue;
            }

            // Check for menu entry (starts with /)
            if line.starts_with('/') {
                let entry = self.parse_entry()?;
                config.entries.push(entry);
                continue;
            }

            // Check for macro definition (contains =)
            if line.contains('=') && line.starts_with("${") {
                self.parse_macro_definition(line)?;
                self.current_line += 1;
                continue;
            }

            // Parse global option (key: value)
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = self.macros.expand(line[colon_pos + 1..].trim());

                match key.as_str() {
                    "timeout" => {
                        config.timeout = if value.to_lowercase() == "no" {
                            None
                        } else {
                            Some(
                                value
                                    .parse()
                                    .map_err(|_| BootError::Generic("Invalid timeout"))?,
                            )
                        };
                    },
                    "default_entry" => {
                        config.default_entry = value
                            .parse()
                            .map_err(|_| BootError::Generic("Invalid default_entry"))?;
                    },
                    "quiet" => config.quiet = value.to_lowercase() == "yes",
                    "serial" => config.serial = value.to_lowercase() == "yes",
                    "serial_baudrate" => {
                        config.serial_baudrate = value
                            .parse()
                            .map_err(|_| BootError::Generic("Invalid baudrate"))?;
                    },
                    "verbose" => config.verbose = value.to_lowercase() == "yes",
                    "interface_resolution" => {
                        config.interface_resolution = Self::parse_resolution(&value);
                    },
                    "interface_branding" => {
                        config.interface_branding = Some(value);
                    },
                    "wallpaper" => {
                        config.wallpaper = Some(value);
                    },
                    "wallpaper_style" => {
                        if let Some(style) = WallpaperStyle::from_str(&value) {
                            config.wallpaper_style = style;
                        }
                    },
                    "editor_enabled" => config.editor_enabled = value.to_lowercase() == "yes",
                    _ => {
                        info!("Unknown global option: {}", key);
                    },
                }
            }

            self.current_line += 1;
        }

        Ok(config)
    }

    fn parse_entry(&mut self) -> Result<MenuEntry> {
        let line = &self.lines[self.current_line].clone();

        // Count leading slashes to determine hierarchy level
        let level = line.chars().take_while(|&c| c == '/').count();

        // Check for expanded flag (+)
        let expanded = line.chars().nth(level) == Some('+');
        let name_start = if expanded { level + 1 } else { level };

        let name = line[name_start..].trim().to_string();

        self.current_line += 1;

        // Parse entry options
        let mut entry = MenuEntry::new(name, String::new(), String::new());
        entry.expanded = expanded;

        // Read options until next entry or end
        while self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line].clone();

            // Empty or comment
            if line.is_empty() || line.starts_with('#') {
                self.current_line += 1;
                continue;
            }

            // Next entry (starts with /)
            if line.starts_with('/') {
                // Check if it's a sub-entry
                let sub_level = line.chars().take_while(|&c| c == '/').count();
                if sub_level > level {
                    // Parse sub-entry
                    let sub_entry = self.parse_entry()?;
                    entry.sub_entries.push(sub_entry);
                    continue;
                } else {
                    // Same level or higher - end of this entry
                    break;
                }
            }

            // Parse option
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = self.macros.expand(line[colon_pos + 1..].trim());

                match key.as_str() {
                    "comment" => entry.comment = Some(value),
                    "protocol" => entry.protocol = value,
                    "path" | "kernel_path" => entry.kernel_path = value,
                    "cmdline" | "kernel_cmdline" => entry.cmdline = Some(value),
                    "module_path" => {
                        entry.modules.push(Module {
                            path:    value,
                            cmdline: None,
                        });
                    },
                    "module_string" | "module_cmdline" => {
                        // Apply to last module
                        if let Some(last_mod) = entry.modules.last_mut() {
                            last_mod.cmdline = Some(value);
                        }
                    },
                    "resolution" => {
                        if let Some((w, h, bpp)) = Self::parse_resolution_full(&value) {
                            entry.resolution = Some((w, h, bpp));
                        }
                    },
                    "textmode" => entry.textmode = value.to_lowercase() == "yes",
                    "dtb_path" => entry.dtb_path = Some(value),
                    "kaslr" => entry.kaslr = value.to_lowercase() == "yes",
                    _ => {
                        info!("Unknown entry option: {}", key);
                    },
                }

                self.current_line += 1;
            } else {
                // Invalid line
                self.current_line += 1;
            }
        }

        Ok(entry)
    }

    fn parse_macro_definition(&mut self, line: &str) -> Result<()> {
        // Format: ${MACRO_NAME}=value
        if let Some(eq_pos) = line.find('=') {
            let macro_part = &line[..eq_pos];
            let value = line[eq_pos + 1..].trim();

            // Extract macro name from ${NAME}
            if macro_part.starts_with("${") && macro_part.ends_with('}') {
                let name = macro_part[2..macro_part.len() - 1].to_string();
                self.macros.define(name, value.to_string());
            }
        }

        Ok(())
    }

    fn parse_resolution(s: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() >= 2 {
            let width = parts[0].parse().ok()?;
            let height = parts[1].parse().ok()?;
            Some((width, height))
        } else {
            None
        }
    }

    fn parse_resolution_full(s: &str) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() >= 2 {
            let width = parts[0].parse().ok()?;
            let height = parts[1].parse().ok()?;
            let bpp = if parts.len() >= 3 {
                parts[2].parse().ok()?
            } else {
                32 // Default BPP
            };
            Some((width, height, bpp))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let config_text = r#"
timeout: 5
default_entry: 1

/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    cmdline: quiet
"#;

        let config = ConfigParser::parse(config_text).unwrap();
        assert_eq!(config.timeout, Some(5));
        assert_eq!(config.default_entry, 1);
        assert_eq!(config.entries.len(), 1);
        assert_eq!(config.entries[0].name, "Redstone OS");
        assert_eq!(config.entries[0].protocol, "limine");
    }

    #[test]
    fn test_parse_with_macros() {
        let config_text = r#"
${OS_NAME}=Redstone

/Bootstrap ${OS_NAME}
    protocol: limine
    kernel_path: boot():/forge
"#;

        let config = ConfigParser::parse(config_text).unwrap();
        assert_eq!(config.entries[0].name, "Bootstrap Redstone");
    }
}
