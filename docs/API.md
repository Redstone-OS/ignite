# Referência de API - Ignite Bootloader

Este documento fornece a referência completa da API pública do Ignite Bootloader.

## 📋 Módulos Principais

### `ignite::config`

Sistema de configuração do bootloader.

#### Tipos Principais

**`BootConfig`** - Configuração global
```rust
pub struct BootConfig {
    pub timeout: Option<u32>,
    pub default_entry_idx: usize,
    pub quiet: bool,
    pub serial_enabled: bool,
    pub resolution: Option<(u32, u32)>,
    pub wallpaper: Option<String>,
    pub entries: Vec<Entry>,
}

impl BootConfig {
    pub fn recovery() -> Self;  // Configuração de emergência
}
```

**`Entry`** - Entrada de boot
```rust
pub struct Entry {
    pub name: String,
    pub protocol: Protocol,
    pub path: String,
    pub cmdline: Option<String>,
    pub modules: Vec<Module>,
    pub dtb_path: Option<String>,
}
```

**`Protocol`** - Protocolos de boot suportados
```rust
pub enum Protocol {
    Linux,        // Linux Boot Protocol
    Limine,       // Protocolo nativo (Redstone/Limine)
    EfiChainload, // UEFI LoadImage/StartImage
    Multiboot2,   // Multiboot2 Specification
    Unknown,
}

impl From<&str> for Protocol;
```

#### Funções

**`loader::load_configuration`**
```rust
pub fn load_configuration(fs: &mut dyn FileSystem) -> Result<BootConfig>
```

Carrega `ignite.conf` do disco e retorna configuração parseada.

---

### `ignite::memory`

Gerenciamento de memória e paging.

#### Traits

**`FrameAllocator`**
```rust
pub trait FrameAllocator {
    fn allocate_frame(&mut self, count: usize) -> Result<u64>;
    fn free_frame(&mut self, addr: u64, count: usize) -> Result<()>;
}
```

Abstração para alocação de frames físicos.

#### Tipos

**`BumpAllocator`** - Alocador de heap
```rust
pub struct BumpAllocator;

impl BumpAllocator {
    pub const fn new() -> Self;
    pub unsafe fn init(&self, start: usize, size: usize);
}

unsafe impl GlobalAlloc for BumpAllocator;
```

**`PageTableManager`** - Gerenciador de paging
```rust
pub struct PageTableManager;

impl PageTableManager {
    pub fn new(allocator: &mut impl FrameAllocator) -> Result<Self>;
    pub fn pml4_addr(&self) -> u64;
    pub fn identity_map(&mut self, phys: u64, count: usize, allocator: &mut impl FrameAllocator) -> Result<()>;
    pub fn map_kernel(&mut self, phys: u64, virt: u64, pages: usize, allocator: &mut impl FrameAllocator) -> Result<()>;
}
```

**`UefiFrameAllocator`** - Alocador via UEFI
```rust
pub struct UefiFrameAllocator;

impl UefiFrameAllocator {
    pub fn new(boot_services: &BootServices) -> Self;
}

impl FrameAllocator for UefiFrameAllocator;
```

---

### `ignite::fs`

Sistemas de arquivos.

#### Traits

**`FileSystem`**
```rust
pub trait FileSystem {
    fn root(&mut self) -> Result<Box<dyn DirectoryHandle>>;
}
```

**`FileHandle`**
```rust
pub trait FileHandle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn size(&self) -> u64;
}
```

**`DirectoryHandle`**
```rust
pub trait DirectoryHandle {
    fn open_file(&mut self, path: &str) -> Result<Box<dyn FileHandle>>;
}
```

#### Tipos

**`UefiFileSystem`**
```rust
pub struct UefiFileSystem;

impl UefiFileSystem {
    pub fn new(proto: &mut SimpleFileSystemProtocol) -> Self;
}

impl FileSystem for UefiFileSystem;
```

#### Funções Auxiliares

**`read_to_bytes`**
```rust
pub fn read_to_bytes(file: &mut dyn FileHandle) -> Result<Vec<u8>>
```

---

### `ignite::video`

Subsistema de vídeo e framebuffer.

#### Tipos

**`FramebufferInfo`**
```rust
pub struct FramebufferInfo {
    pub addr: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}
```

**`PixelFormat`**
```rust
pub enum PixelFormat {
    Rgb,     // Red-Green-Blue
    Bgr,     // Blue-Green-Red
    Bitmask, // Custom bitmask
}
```

#### Funções

**`init_video`**
```rust
pub fn init_video(bs: &BootServices) -> Result<(GraphicsOutputProtocol, FramebufferInfo)>
```

Inicializa GOP e retorna protocolo e informações do framebuffer.

---

### `ignite::uefi`

Interface UEFI (FFI).

#### Tipos Globais

**`Handle`** - Handle opaco do UEFI
```rust
#[repr(transparent)]
pub struct Handle(*mut core::ffi::c_void);

impl Handle {
    pub fn null() -> Self;
}
```

**`Status`** - Código de retorno UEFI
```rust
#[repr(transparent)]
pub struct Status(usize);

impl Status {
    pub const SUCCESS: Status = Status(0);
    pub const fn is_error(self) -> bool;
    pub fn to_result(self) -> Result<()>;
}
```

**`SystemTable`**
```rust
#[repr(C)]
pub struct SystemTable {
    pub hdr: TableHeader,
    pub firmware_vendor: *const u16,
    pub firmware_revision: u32,
    pub console_in_handle: Handle,
    pub con_in: *mut SimpleTextInputProtocol,
    pub console_out_handle: Handle,
    pub con_out: *mut SimpleTextOutputProtocol,
    // ...
}

impl SystemTable {
    pub fn boot_services(&self) -> &BootServices;
    pub fn runtime_services(&self) -> &RuntimeServices;
}
```

**`BootServices`**
```rust
#[repr(C)]
pub struct BootServices {
    pub hdr: TableHeader,
    // ...
}

impl BootServices {
    pub fn allocate_pool(&self, ty: MemoryType, size: usize) -> Result<*mut u8>;
    pub fn free_pool(&self, buffer: *mut u8) -> Result<()>;
    pub fn open_protocol(&self, handle: Handle, guid: &Guid, agent: Handle, controller: Handle, attributes: u32) -> Result<*mut core::ffi::c_void>;
    pub fn exit_boot_services(&self, image_handle: Handle, map_key: usize) -> Status;
}
```

---

### `ignite::protos`

Protocolos de boot.

#### Trait

**`BootProtocol`**
```rust
pub trait BootProtocol {
    fn name(&self) -> &str;
    fn identify(&self, data: &[u8]) -> bool;
    fn load(&mut self, data: &[u8], cmdline: Option<&str>, modules: Vec<LoadedFile>) -> Result<KernelLaunchInfo>;
}
```

#### Tipos

**`KernelLaunchInfo`**
```rust
pub struct KernelLaunchInfo {
    pub entry_point: u64,
    pub stack_pointer: Option<u64>,
    pub rdi: u64,  // 1º argumento
    pub rsi: u64,  // 2º argumento
    pub rdx: u64,  // 3º argumento
    pub rbx: u64,  // Multiboot2
}
```

#### Funções

**`load_any`**
```rust
pub fn load_any(
    allocator: &mut dyn FrameAllocator,
    page_table: &mut PageTableManager,
    kernel_data: &[u8],
    cmdline: Option<&str>,
    modules: Vec<LoadedFile>,
) -> Result<KernelLaunchInfo>
```

Detecta automaticamente protocolo e carrega kernel.

---

### `ignite::security`

Segurança e validação.

#### Tipos

**`SecurityPolicy`**
```rust
pub struct SecurityPolicy {
    require_secure_boot: bool,
    require_tpm: bool,
    on_fail: PolicyAction,
}

impl SecurityPolicy {
    pub fn new(config: &BootConfig) -> Self;
}
```

**`SecureBootState`**
```rust
pub enum SecureBootState {
    Enabled,
    Disabled,
    SetupMode,
}
```

#### Funções

**`validate_and_measure`**
```rust
pub fn validate_and_measure(data: &[u8], name: &str, policy: &SecurityPolicy) -> Result<()>
```

**`tpm::measure_binary`**
```rust
pub fn measure_binary(data: &[u8], pcr: u32, description: &str) -> Result<()>
```

---

### `ignite::ui`

Interface de usuário.

#### Tipos

**`Menu`**
```rust
pub struct Menu<'a>;

impl<'a> Menu<'a> {
    pub fn new(config: &'a BootConfig) -> Self;
    pub unsafe fn run(&mut self, fb_ptr: u64, fb_info: FramebufferInfo) -> &'a Entry;
}
```

---

### `ignite::core`

Tipos centrais e utilitários.

#### Tipos de Erro

**`BootError`**
```rust
pub enum BootError {
    Memory(MemoryError),
    Io(IoError),
    Config(ConfigError),
    Security(SecurityError),
    Uefi(Status),
    Generic(&'static str),
}
```

**`Result<T>`**
```rust
pub type Result<T> = core::result::Result<T, BootError>;
```

#### Handoff

**`BootInfo`**
```rust
#[repr(C)]
pub struct BootInfo {
    pub magic: u64,
    pub framebuffer: FramebufferInfo,
    pub memory_map: *const MemoryMap,
    pub rsdp: u64,
    pub kernel_cmdline: *const u8,
    pub initrd: Option<Module>,
    pub modules: &'static [Module],
}
```

---

## Macros

**`println!`**
```rust
println!("Mensagem de log: {}", valor);
```

Escreve na saída serial (COM1).

---

## Constantes de Configuração

```rust
// core/config.rs
pub mod memory {
    pub const BOOTLOADER_HEAP_SIZE: usize = 2 * 1024 * 1024; // 2 MiB
    pub const KERNEL_HIGHER_HALF: u64 = 0xFFFFFFFF80000000;
}
```

---

**Última Atualização**: 2025-12-21  
**Versão**: 1.0
