//! Path Resolution System
//!
//! Handles different path formats:
//! - boot():/path
//! - boot(2):/path
//! - hdd(1:2):/path
//! - guid(UUID):/path
//! - fslabel(LABEL):/path

use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt;

/// Path resource type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResource {
    /// boot() or boot(N) - Boot partition
    Boot(Option<u32>),

    /// hdd(D:P) - Hard disk drive and partition
    Hdd(u32, Option<u32>),

    /// odd(D:P) - Optical disk drive
    Odd(u32, Option<u32>),

    /// guid(UUID) or uuid(UUID) - Partition GUID
    Guid(String),

    /// fslabel(LABEL) - Filesystem label
    FsLabel(String),

    /// tftp(IP) - Network TFTP (PXE только)
    Tftp(Option<String>),
}

impl PathResource {
    /// Parse resource from string like "boot(2)" or "guid(xxx)"
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
                        // Parse "D:P" format
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

/// Complete path with resource and file path
#[derive(Debug, Clone)]
pub struct Path {
    /// Resource (boot device, partition, etc.)
    pub resource: PathResource,

    /// File path on the resource
    pub path: String,

    /// Optional BLAKE2B hash for verification
    pub hash: Option<String>,
}

impl Path {
    /// Parse a full path like "boot(2):/kernel#hash"
    pub fn parse(s: &str) -> Option<Self> {
        // Check for hash at the end
        let (path_part, hash) = if let Some(hash_pos) = s.rfind('#') {
            let hash = s[hash_pos + 1..].to_string();
            (&s[..hash_pos], Some(hash))
        } else {
            (s, None)
        };

        // Find ":" separator
        let colon_pos = path_part.find(':')?;

        // Parse resource
        let (resource, consumed) = PathResource::parse(path_part)?;

        if consumed != colon_pos {
            return None; // Resource didn't end at colon
        }

        // Get file path (skip the colon)
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
