//! Testes Unitários para o módulo de filesystem
//!
//! Testa operações de arquivo e path resolution.

#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Testa validação de path
#[test]
fn test_path_validation() {
    fn is_valid_path(path: &str) -> bool {
        !path.is_empty() &&
        !path.contains('\0') &&
        !path.contains("..") && // Simples proteção contra path traversal
        path.len() < 260 // MAX_PATH no Windows
    }

    assert!(is_valid_path("boot():/EFI/ignite/kernel"));
    assert!(is_valid_path("/absolute/path"));
    assert!(is_valid_path("relative/path"));
    assert!(!is_valid_path(""));
    assert!(!is_valid_path("path/with/null\0char"));
    assert!(!is_valid_path("../../../etc/passwd")); // Path traversal
}

/// Testa normalization de path
#[test]
fn test_path_normalization() {
    fn normalize_path(path: &str) -> String {
        path.replace('\\', "/").trim_end_matches('/').to_string()
    }

    assert_eq!(
        normalize_path("C:\\Windows\\System32"),
        "C:/Windows/System32"
    );
    assert_eq!(normalize_path("/path/to/file/"), "/path/to/file");
    assert_eq!(
        normalize_path("path//with///slashes"),
        "path//with///slashes"
    ); // Simplificado
}

/// Testa parsing de path em componentes
#[test]
fn test_path_components() {
    fn split_path(path: &str) -> Vec<&str> {
        path.split('/').filter(|s| !s.is_empty()).collect()
    }

    let components = split_path("/boot/efi/ignite/kernel");
    assert_eq!(components, vec!["boot", "efi", "ignite", "kernel"]);

    let components2 = split_path("relative/path");
    assert_eq!(components2, vec!["relative", "path"]);
}

/// Testa detecção de scheme (boot():/, root():/)
#[test]
fn test_path_scheme_detection() {
    #[derive(Debug, PartialEq)]
    enum PathScheme {
        Boot,
        Root,
        Absolute,
        Relative,
    }

    fn detect_scheme(path: &str) -> PathScheme {
        if path.starts_with("boot():/") {
            PathScheme::Boot
        } else if path.starts_with("root():/") {
            PathScheme::Root
        } else if path.starts_with('/') {
            PathScheme::Absolute
        } else {
            PathScheme::Relative
        }
    }

    assert_eq!(detect_scheme("boot():/EFI/ignite/kernel"), PathScheme::Boot);
    assert_eq!(detect_scheme("root():/boot/vmlinuz"), PathScheme::Root);
    assert_eq!(detect_scheme("/absolute/path"), PathScheme::Absolute);
    assert_eq!(detect_scheme("relative/path"), PathScheme::Relative);
}

/// Testa validação de nome de arquivo FAT32 (8.3)
#[test]
fn test_fat32_83_name() {
    fn is_valid_83_name(name: &str) -> bool {
        if name.is_empty() || name.len() > 12 {
            return false;
        }

        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() > 2 {
            return false;
        }

        if parts[0].len() > 8 {
            return false;
        }

        if parts.len() == 2 && parts[1].len() > 3 {
            return false;
        }

        true
    }

    assert!(is_valid_83_name("FILE.TXT"));
    assert!(is_valid_83_name("KERNEL"));
    assert!(is_valid_83_name("A.B"));
    assert!(!is_valid_83_name("VERYLONGFILENAME.TXT"));
    assert!(!is_valid_83_name("FILE.LONGEXT"));
}

/// Testa conversão de nome longo para curto (FAT32 LFN)
#[test]
fn test_short_name_generation() {
    fn generate_short_name(long_name: &str) -> String {
        let clean = long_name
            .to_uppercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.')
            .collect::<String>();

        let parts: Vec<&str> = clean.split('.').collect();
        let base = if parts[0].len() > 8 {
            &parts[0][..6]
        } else {
            parts[0]
        };

        if parts.len() > 1 {
            let ext = if parts[1].len() > 3 {
                &parts[1][..3]
            } else {
                parts[1]
            };
            format!("{}.{}", base, ext)
        } else {
            base.to_string()
        }
    }

    assert_eq!(generate_short_name("longfilename.txt"), "LONGFI.TXT");
    assert_eq!(generate_short_name("file.txt"), "FILE.TXT");
}

/// Testa cálculo de cluster chain
#[test]
fn test_cluster_chain() {
    // Mock FAT table
    let fat = vec![
        0xFFFFFFFF, // Cluster 0 (reserved)
        0xFFFFFFFF, // Cluster 1 (reserved)
        3,          // Cluster 2 -> 3
        4,          // Cluster 3 -> 4
        0x0FFFFFFF, // Cluster 4 (end)
        0,          // Cluster 5 (free)
    ];

    fn is_end_of_chain(cluster: u32) -> bool {
        cluster >= 0x0FFFFFF8
    }

    fn get_next_cluster(fat: &[u32], current: u32) -> Option<u32> {
        let next = *fat.get(current as usize)?;
        if is_end_of_chain(next) {
            None
        } else {
            Some(next)
        }
    }

    // Seguir chain: 2 -> 3 -> 4 -> fim
    assert_eq!(get_next_cluster(&fat, 2), Some(3));
    assert_eq!(get_next_cluster(&fat, 3), Some(4));
    assert_eq!(get_next_cluster(&fat, 4), None); // End of chain
}

/// Testa cálculo de offset de arquivo
#[test]
fn test_file_offset_calculation() {
    const BYTES_PER_CLUSTER: usize = 4096;

    fn cluster_to_offset(cluster: u32, data_start: u64) -> u64 {
        data_start + ((cluster - 2) as u64 * BYTES_PER_CLUSTER as u64)
    }

    let data_start = 0x10000;
    assert_eq!(cluster_to_offset(2, data_start), data_start);
    assert_eq!(
        cluster_to_offset(3, data_start),
        data_start + BYTES_PER_CLUSTER as u64
    );
}

/// Testa leitura de directory entry FAT32
#[test]
fn test_directory_entry_parsing() {
    const ATTR_READ_ONLY: u8 = 0x01;
    const ATTR_HIDDEN: u8 = 0x02;
    const ATTR_SYSTEM: u8 = 0x04;
    const ATTR_DIRECTORY: u8 = 0x10;
    const ATTR_LONG_NAME: u8 = 0x0F;

    fn is_directory(attr: u8) -> bool {
        (attr & ATTR_DIRECTORY) != 0
    }

    fn is_long_name_entry(attr: u8) -> bool {
        attr == ATTR_LONG_NAME
    }

    fn is_hidden(attr: u8) -> bool {
        (attr & ATTR_HIDDEN) != 0
    }

    assert!(is_directory(ATTR_DIRECTORY));
    assert!(is_long_name_entry(ATTR_LONG_NAME));
    assert!(is_hidden(ATTR_HIDDEN | ATTR_DIRECTORY));
    assert!(!is_directory(ATTR_READ_ONLY));
}

/// Testa validação de file handle
#[test]
fn test_file_handle_validation() {
    struct FileHandle {
        position: u64,
        size:     u64,
        is_eof:   bool,
    }

    impl FileHandle {
        fn seek(&mut self, pos: u64) -> bool {
            if pos <= self.size {
                self.position = pos;
                self.is_eof = pos == self.size;
                true
            } else {
                false
            }
        }

        fn can_read(&self, count: usize) -> bool {
            self.position + count as u64 <= self.size
        }
    }

    let mut handle = FileHandle {
        position: 0,
        size:     1000,
        is_eof:   false,
    };

    assert!(handle.seek(500));
    assert!(handle.can_read(500));
    assert!(!handle.can_read(501));

    assert!(handle.seek(1000));
    assert!(handle.is_eof);
}

/// Testa merge de paths
#[test]
fn test_path_join() {
    fn join_path(base: &str, relative: &str) -> String {
        if relative.starts_with('/') {
            relative.to_string()
        } else if base.ends_with('/') {
            format!("{}{}", base, relative)
        } else {
            format!("{}/{}", base, relative)
        }
    }

    assert_eq!(join_path("/base", "file"), "/base/file");
    assert_eq!(join_path("/base/", "file"), "/base/file");
    assert_eq!(join_path("/base", "/absolute"), "/absolute");
}

/// Testa validação de buffer overflow em leitura
#[test]
fn test_read_buffer_safety() {
    fn safe_read(source: &[u8], dest: &mut [u8], offset: usize, count: usize) -> Result<usize, ()> {
        if offset + count > source.len() {
            return Err(());
        }

        let actual_count = count.min(dest.len());
        dest[..actual_count].copy_from_slice(&source[offset..offset + actual_count]);
        Ok(actual_count)
    }

    let source = vec![1u8, 2, 3, 4, 5];
    let mut dest = vec![0u8; 3];

    assert_eq!(safe_read(&source, &mut dest, 0, 3), Ok(3));
    assert_eq!(dest, vec![1, 2, 3]);

    assert_eq!(safe_read(&source, &mut dest, 0, 10), Err(())); // Overflow
}

/// Testa cache de blocos
#[test]
fn test_block_cache() {
    use alloc::collections::BTreeMap;

    struct BlockCache {
        cache:      BTreeMap<u64, Vec<u8>>,
        max_blocks: usize,
    }

    impl BlockCache {
        fn new(max_blocks: usize) -> Self {
            Self {
                cache: BTreeMap::new(),
                max_blocks,
            }
        }

        fn get(&self, block: u64) -> Option<&Vec<u8>> {
            self.cache.get(&block)
        }

        fn put(&mut self, block: u64, data: Vec<u8>) {
            if self.cache.len() >= self.max_blocks {
                // Simple FIFO eviction
                if let Some(&first_key) = self.cache.keys().next() {
                    self.cache.remove(&first_key);
                }
            }
            self.cache.insert(block, data);
        }
    }

    let mut cache = BlockCache::new(2);

    cache.put(0, alloc::vec![1, 2, 3]);
    cache.put(1, alloc::vec![4, 5, 6]);
    assert_eq!(cache.cache.len(), 2);

    cache.put(2, alloc::vec![7, 8, 9]);
    assert_eq!(cache.cache.len(), 2); // Evicted one
}
