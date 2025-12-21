//! Sistema de Resolução de Caminhos
//!
//! Lida com diferentes formatos de caminho:
//! - boot():/path
//! - boot(2):/path
//! - hdd(1:2):/path
//! - guid(UUID):/path
//! - fslabel(LABEL):/path

use alloc::string::{String, ToString};
use core::fmt;

/// Tipo de recurso de caminho
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResource {
    /// boot() ou boot(N) - Partição de boot
    Boot(Option<u32>),

    /// hdd(D:P) - Disco rígido e partição
    Hdd(u32, Option<u32>),

    /// odd(D:P) - Unidade de disco óptico
    Odd(u32, Option<u32>),

    /// guid(UUID) ou uuid(UUID) - GUID da partição
    Guid(String),

    /// fslabel(LABEL) - Rótulo do sistema de arquivos
    FsLabel(String),

    /// tftp(IP) - Rede TFTP (PXE apenas)
    Tftp(Option<String>),
}

impl PathResource {
    /// Parsear recurso de string como "boot(2)" ou "guid(xxx)"
    pub fn parse(s: &str) -> Option<(Self, usize)> {
        if let Some(paren_pos) = s.find('(') {
            let resource_type = &s[..paren_pos];
            if let Some(close_paren) = s[paren_pos..].find(')') {
                let arg = &s[paren_pos + 1..paren_pos + close_paren];
                let consumed = paren_pos + close_paren + 1;

                let resource = match resource_type.to_lowercase().as_str() {
                    "boot" => {
                        let partition = if arg.is_empty() {
                            None
                        } else {
                            arg.parse::<u32>().ok()
                        };
                        Some(PathResource::Boot(partition))
                    },
                    "hdd" => {
                        // Parsear formato "D:P"
                        if let Some(colon_pos) = arg.find(':') {
                            let drive = arg[..colon_pos].parse::<u32>().ok()?;
                            let partition = if arg[colon_pos + 1..].is_empty() {
                                None
                            } else {
                                arg[colon_pos + 1..].parse::<u32>().ok()
                            };
                            Some(PathResource::Hdd(drive, partition))
                        } else {
                            None
                        }
                    },
                    "odd" => {
                        if let Some(colon_pos) = arg.find(':') {
                            let drive = arg[..colon_pos].parse::<u32>().ok()?;
                            let partition = if arg[colon_pos + 1..].is_empty() {
                                None
                            } else {
                                arg[colon_pos + 1..].parse::<u32>().ok()
                            };
                            Some(PathResource::Odd(drive, partition))
                        } else {
                            None
                        }
                    },
                    "guid" | "uuid" => Some(PathResource::Guid(arg.to_string())),
                    "fslabel" => Some(PathResource::FsLabel(arg.to_string())),
                    "tftp" => {
                        let ip = if arg.is_empty() {
                            None
                        } else {
                            Some(arg.to_string())
                        };
                        Some(PathResource::Tftp(ip))
                    },
                    _ => None,
                };

                return resource.map(|r| (r, consumed));
            }
        }

        None
    }
}

impl fmt::Display for PathResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathResource::Boot(None) => write!(f, "boot()"),
            PathResource::Boot(Some(n)) => write!(f, "boot({})", n),
            PathResource::Hdd(d, None) => write!(f, "hdd({}:)", d),
            PathResource::Hdd(d, Some(p)) => write!(f, "hdd({}:{})", d, p),
            PathResource::Odd(d, None) => write!(f, "odd({}:)", d),
            PathResource::Odd(d, Some(p)) => write!(f, "odd({}:{})", d, p),
            PathResource::Guid(g) => write!(f, "guid({})", g),
            PathResource::FsLabel(l) => write!(f, "fslabel({})", l),
            PathResource::Tftp(None) => write!(f, "tftp()"),
            PathResource::Tftp(Some(ip)) => write!(f, "tftp({})", ip),
        }
    }
}

/// Caminho completo com recurso e caminho de arquivo
#[derive(Debug, Clone)]
pub struct Path {
    /// Recurso (dispositivo de boot, partição, etc.)
    pub resource: PathResource,

    /// Caminho do arquivo no recurso
    pub path: String,

    /// Hash BLAKE2B opcional para verificação
    pub hash: Option<String>,
}

impl Path {
    /// Parsear um caminho completo como "boot(2):/kernel#hash"
    pub fn parse(s: &str) -> Option<Self> {
        // Verificar hash no final
        let (path_part, hash) = if let Some(hash_pos) = s.rfind('#') {
            let hash = s[hash_pos + 1..].to_string();
            (&s[..hash_pos], Some(hash))
        } else {
            (s, None)
        };

        // Encontrar separador ":"
        let colon_pos = path_part.find(':')?;

        // Parsear recurso
        let (resource, consumed) = PathResource::parse(path_part)?;

        if consumed != colon_pos {
            return None; // Recurso não terminou nos dois pontos
        }

        // Obter caminho do arquivo (pular os dois pontos)
        let file_path = path_part[colon_pos + 1..].to_string();

        Some(Path {
            resource,
            path: file_path,
            hash,
        })
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.resource, self.path)?;
        if let Some(ref hash) = self.hash {
            write!(f, "#{}", hash)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boot_default() {
        let path = Path::parse("boot():/kernel").unwrap();
        assert!(matches!(path.resource, PathResource::Boot(None)));
        assert_eq!(path.path, "/kernel");
        assert!(path.hash.is_none());
    }

    #[test]
    fn test_parse_boot_partition() {
        let path = Path::parse("boot(2):/vmlinuz").unwrap();
        assert!(matches!(path.resource, PathResource::Boot(Some(2))));
        assert_eq!(path.path, "/vmlinuz");
    }

    #[test]
    fn test_parse_with_hash() {
        let path = Path::parse("boot():/kernel#abc123").unwrap();
        assert_eq!(path.hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_parse_hdd() {
        let path = Path::parse("hdd(1:2):/boot/kernel").unwrap();
        assert!(matches!(path.resource, PathResource::Hdd(1, Some(2))));
        assert_eq!(path.path, "/boot/kernel");
    }
}
