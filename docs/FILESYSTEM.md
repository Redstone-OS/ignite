# Sistemas de Arquivos - Ignite Bootloader

Documentação do subsistema de sistemas de arquivos.

## 📋 Sistemas Suportados

- [FAT32](#fat32)
- [UEFI Simple File System](#uefi-simple-file-system)
- [Virtual File System (VFS)](#virtual-file-system-vfs)
- [Path Resolution](#path-resolution)

---

## FAT32

### Especificação

**File Allocation Table 32-bit** - Sistema de arquivos padrão para ESP.

**Características**:
- Tamanho máximo de arquivo: 4 GB
- Tamanho máximo de volume: 2 TB (teórico, 32 GB prático)
- Nomes de arquivo: 8.3 (curtos) ou LFN (longos)
- Case-insensitive

---

### Driver Nativo

O Ignite possui driver FAT32 básico (somente leitura):

```rust
pub struct Fat32FileSystem {
    boot_sector: BootSector,
    fat: Vec<u32>,
    root_cluster: u32,
    device: Box<dyn BlockDevice>,
}

impl Fat32FileSystem {
    pub fn new(device: Box<dyn BlockDevice>) -> Result<Self> {
        // Ler boot sector
        let boot_sector = BootSector::read(&device)?;
        
        // Validar assinatura
        assert_eq!(boot_sector.signature, 0xAA55);
        
        // Ler FAT
        let fat = read_fat(&device, &boot_sector)?;
        
        Ok(Self {
            boot_sector,
            fat,
            root_cluster: boot_sector.root_cluster,
            device,
        })
    }
}
```

---

### Boot Sector

```rust
#[repr(C, packed)]
pub struct BootSector {
    pub jmp_boot: [u8; 3],
    pub oem_name: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub root_entry_count: u16,      // 0 para FAT32
    pub total_sectors_16: u16,       // 0 para FAT32
    pub media: u8,
    pub fat_size_16: u16,            // 0 para FAT32
    pub sectors_per_track: u16,
    pub num_heads: u16,
    pub hidden_sectors: u32,
    pub total_sectors_32: u32,
    
    // FAT32-specific
    pub fat_size_32: u32,
    pub ext_flags: u16,
    pub fs_version: u16,
    pub root_cluster: u32,
    pub fs_info: u16,
    pub backup_boot_sector: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub reserved1: u8,
    pub boot_signature: u8,
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    pub fs_type: [u8; 8],            // "FAT32   "
    pub boot_code: [u8; 420],
    pub signature: u16,               // 0xAA55
}
```

---

### Directory Entry

```rust
#[repr(C, packed)]
pub struct DirectoryEntry {
    pub name: [u8; 11],               // 8.3 format
    pub attr: u8,
    pub reserved: u8,
    pub create_time_tenth: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub last_access_date: u16,
    pub first_cluster_high: u16,
    pub write_time: u16,
    pub write_date: u16,
    pub first_cluster_low: u16,
    pub file_size: u32,
}

// Atributos
const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = 0x0F;  // LFN entry
```

---

### Operações

```rust
impl FileSystem for Fat32FileSystem {
    fn root(&mut self) -> Result<Box<dyn DirectoryHandle>> {
        Ok(Box::new(Fat32Directory {
            fs: self,
            cluster: self.root_cluster,
        }))
    }
}

impl DirectoryHandle for Fat32Directory {
    fn open_file(&mut self, path: &str) -> Result<Box<dyn FileHandle>> {
        // Parse path
        let components: Vec<&str> = path.split('/').collect();
        
        // Navigate directories
        let mut current_cluster = self.cluster;
        for (i, component) in components.iter().enumerate() {
            if i == components.len() - 1 {
                // Last component = file
                return self.find_file(current_cluster, component);
            } else {
                // Navigate to subdirectory
                current_cluster = self.find_directory(current_cluster, component)?;
            }
        }
        
        Err(BootError::Io(IoError::NotFound))
    }
}
```

---

## UEFI Simple File System

### SimpleFileSystemProtocol

Interface UEFI padrão para acesso a arquivos:

```rust
#[repr(C)]
pub struct SimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume: extern "efiapi" fn(
        this: *mut SimpleFileSystemProtocol,
        root: *mut *mut FileProtocol,
    ) -> Status,
}

#[repr(C)]
pub struct FileProtocol {
    pub revision: u64,
    pub open: extern "efiapi" fn(...) -> Status,
    pub close: extern "efiapi" fn(...) -> Status,
    pub delete: extern "efiapi" fn(...) -> Status,
    pub read: extern "efiapi" fn(...) -> Status,
    pub write: extern "efiapi" fn(...) -> Status,
    pub get_position: extern "efiapi" fn(...) -> Status,
    pub set_position: extern "efiapi" fn(...) -> Status,
    pub get_info: extern "efiapi" fn(...) -> Status,
    pub set_info: extern "efiapi" fn(...) -> Status,
    pub flush: extern "efiapi" fn(...) -> Status,
}
```

---

### UefiFileSystem Wrapper

```rust
pub struct UefiFileSystem {
    proto: *mut SimpleFileSystemProtocol,
}

impl UefiFileSystem {
    pub fn new(proto: *mut SimpleFileSystemProtocol) -> Self {
        Self { proto }
    }
}

impl FileSystem for UefiFileSystem {
    fn root(&mut self) -> Result<Box<dyn DirectoryHandle>> {
        let mut root_proto: *mut FileProtocol = null_mut();
        
        let status = unsafe {
            ((*self.proto).open_volume)(self.proto, &mut root_proto)
        };
        
        status.to_result()?;
        
        Ok(Box::new(UefiDirectory { proto: root_proto }))
    }
}
```

---

## Virtual File System (VFS)

### Abstração Multi-FS

VFS permite montar múltiplos sistemas de arquivos:

```rust
pub struct VirtualFileSystem {
    mounts: HashMap<String, Box<dyn FileSystem>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            mounts: HashMap::new(),
        }
    }
    
    pub fn mount(&mut self, path: &str, fs: Box<dyn FileSystem>) {
        self.mounts.insert(path.to_string(), fs);
    }
    
    pub fn open(&mut self, path: &str) -> Result<Box<dyn FileHandle>> {
        // Parse path para encontrar mount point
        for (mount_point, fs) in &mut self.mounts {
            if path.starts_with(mount_point) {
                let relative_path = &path[mount_point.len()..];
                return fs.open(relative_path);
            }
        }
        
        Err(BootError::Io(IoError::NotFound))
    }
}
```

---

### Exemplo de Uso

```rust
let mut vfs = VirtualFileSystem::new();

// Montar ESP em boot():/
vfs.mount("boot():", Box::new(uefi_fs));

// Montar partição raiz em root():/
vfs.mount("root():", Box::new(ext4_fs));  // Futuro

// Abrir arquivo
let file = vfs.open("boot():/EFI/ignite/forge")?;
```

---

## Path Resolution

### Esquemas de URL

**boot():/** - ESP (EFI System Partition)
```ini
path = "boot():/EFI/ignite/forge"
# Resolve para: \EFI\ignite\forge na ESP
```

**root():/** - Partição raiz do OS
```ini
path = "root():/boot/vmlinuz"
# Resolve para: /boot/vmlinuz no filesystem raiz
```

**Caminho absoluto** - Relativo ao FS atual
```ini
path = "/EFI/ignite/forge"
# Relativo à raiz do FS montado
```

**Caminho relativo** - Relativo ao diretório atual
```ini
path = "kernel/forge"
# Relativo ao diretório de ignite.conf
```

---

### Implementação

```rust
pub fn resolve_path(path: &str, base_fs: &mut dyn FileSystem) -> Result<Box<dyn FileHandle>> {
    if path.starts_with("boot():/") {
        // Usar ESP
        let relative = &path["boot():/".len()..];
        ESP_FS.open(relative)
    } else if path.starts_with("root():/") {
        // Usar root FS
        let relative = &path["root():/".len()..];
        ROOT_FS.open(relative)
    } else if path.starts_with('/') {
        // Absoluto no FS atual
        base_fs.open(path)
    } else {
        // Relativo ao diretório atual
        base_fs.open(&format!("{}/{}", current_dir(), path))
    }
}
```

---

## Operações de Arquivo

### Traits

```rust
pub trait FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn size(&self) -> u64;
}

pub trait DirectoryHandle {
    fn open_file(&mut self, path: &str) -> Result<Box<dyn FileHandle>>;
    fn read_dir(&mut self) -> Result<Vec<DirEntry>>;  // Futuro
}

pub trait FileSystem {
    fn root(&mut self) -> Result<Box<dyn DirectoryHandle>>;
}
```

---

### Helpers

```rust
// Ler arquivo completo na memória
pub fn read_to_bytes(file: &mut dyn FileHandle) -> Result<Vec<u8>> {
    let size = file.size() as usize;
    let mut buffer = vec![0u8; size];
    
    let mut offset = 0;
    while offset < size {
        let n = file.read(&mut buffer[offset..])?;
        if n == 0 {
            break;
        }
        offset += n;
    }
    
    buffer.truncate(offset);
    Ok(buffer)
}

// Verificar se arquivo existe
pub fn exists(fs: &mut dyn FileSystem, path: &str) -> bool {
    let mut root = fs.root().ok()?;
    root.open_file(path).is_ok()
}
```

---

## Caching

### Buffer Cache (Futuro)

Para melhorar performance, implementar cache de blocos:

```rust
pub struct BufferCache {
    blocks: HashMap<u64, Vec<u8>>,  // block_num -> data
    max_blocks: usize,
}

impl BufferCache {
    pub fn read_block(&mut self, device: &mut dyn BlockDevice, block: u64) -> Result<&[u8]> {
        if !self.blocks.contains_key(&block) {
            // Cache miss
            if self.blocks.len() >= self.max_blocks {
                // Evict (LRU ou FIFO)
                self.evict_one();
            }
            
            let data = device.read_block(block)?;
            self.blocks.insert(block, data);
        }
        
        Ok(&self.blocks[&block])
    }
}
```

---

## Limitações Atuais

- ✅ Leitura de arquivos
- ❌ Escrita de arquivos
- ✅ Navegação de diretórios
- ❌ Criação de arquivos/diretórios
- ✅ FAT32
- ❌ ext4, NTFS (futuros)

---

**Última Atualização**: 2025-12-21
