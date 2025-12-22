# Gerenciamento de Memória - Ignite Bootloader

Documentação técnica do subsistema de memória.

## 📋 Componentes

- [Layout de Memória](#layout-de-memória)
- [Alocação de Frames](#alocação-de-frames)
- [Paging x86_64](#paging-x86_64)
- [Memory Map Handoff](#memory-map-handoff)

---

## Layout de Memória

### Memória Física

```
Endereço        Região
0x00000000      +------------------------+
                | Real Mode IVT           | Vetor de interrupções (legado)
0x00000400      +------------------------+
                | BIOS Data Area          |
0x00000500      +------------------------+
                | Livre (< 1 MB)          |
0x00080000      +------------------------+
                | UEFI Firmware           | Código e dados do firmware
0x00100000      +------------------------+ 1 MB
                | Bootloader Code         | ignite.efi carregado aqui
                | (.text, .data, .bss)    |
0x????????      +------------------------+
                | Bootloader Heap         | 2-4 MB alocado dinamicamente
0x????????      +------------------------+
                | Kernel Carregado        | Segmentos PT_LOAD do ELF
0x????????      +------------------------+
                | Page Tables             | PML4, PDPT, PD, PT
0x????????      +------------------------+
                | Módulos (Initrd)        |
0x????????      +------------------------+
                | ACPI Tables             | Fornecidas pelo firmware
0x????????      +------------------------+
                | Framebuffer (VRAM)      | Pode estar >4GB
0x????????      +------------------------+
                | livre                   |
                :                         :
```

---

### Memória Virtual (Kernel)

```
Espaço de Endereço de 48 bits (256 TB teórico)

0x0000_0000_0000_0000  +------------------------+
                       | Userspace               | 128 TB
                       | (não mapeado no boot)   |
0x0000_7FFF_FFFF_FFFF  +------------------------+
                       | Canonical hole          | Causar #GP se acessado
0xFFFF_8000_0000_0000  +------------------------+
                       | Direct Map              | 128 TB
                       | (toda RAM física)       | Offset: +0xFFFF800000000000
0xFFFF_87FF_FFFF_FFFF  +------------------------+
                       | Não mapeado             |
0xFFFFFFFF_8000_0000   +------------------------+ -2 GB
                       | Kernel .text            | Código do kernel
0xFFFFFFFF_9000_0000   +------------------------+
                       | Kernel .data/.bss       |
0xFFFFFFFF_A000_0000   +------------------------+
                       | Kernel Heap             |
0xFFFFFFFF_B000_0000   +------------------------+
                       | Kernel Stacks           |
0xFFFFFFFF_C000_0000   +------------------------+
                       | MMIO / Devices          |
0xFFFFFFFF_FFFF_FFFF   +------------------------+
```

---

## Alocação de Frames

### FrameAllocator Trait

Abstração para alocação de frames físicos (páginas de 4 KB):

```rust
pub trait FrameAllocator {
    fn allocate_frame(&mut self, count: usize) -> Result<u64>;
    fn free_frame(&mut self, addr: u64, count: usize) -> Result<()>;
}
```

---

### UefiFrameAllocator

Usa `AllocatePages()` do UEFI Boot Services:

```rust
pub struct UefiFrameAllocator<'a> {
    boot_services: &'a BootServices,
}

impl FrameAllocator for UefiFrameAllocator<'_> {
    fn allocate_frame(&mut self, count: usize) -> Result<u64> {
        let pages = count; // 1 frame = 1 page (4 KB)
        let addr = self.boot_services.allocate_pages(
            AllocateType::AllocateAnyPages,
            MemoryType::LoaderData,
            pages,
        )?;
        Ok(addr)
    }
    
    fn free_frame(&mut self, addr: u64, count: usize) -> Result<()> {
        self.boot_services.free_pages(addr, count)
    }
}
```

**Vantagens**:
- Simples
- Firmware garante que frames são válidos

**Desvantagens**:
- Só funciona antes de `ExitBootServices()`
- Frames podem estar fragmentados

---

### BumpAllocator (Heap)

Alocador linear para heap do bootloader:

```rust
pub struct BumpAllocator {
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
    next: AtomicUsize,
}

impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        
        loop {
            let current = self.next.load(Ordering::Relaxed);
            let aligned = (current + align - 1) & !(align - 1);
            let new = aligned + size;
            
            if new > self.heap_end.load(Ordering::Relaxed) {
                return null_mut(); // OOM
            }
            
            if self.next.compare_exchange_weak(
                current,
                new,
                Ordering::Release,
                Ordering::Relaxed,
            ).is_ok() {
                return aligned as *mut u8;
            }
        }
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator não suporta free individual
        // Memória será liberada quando bootloader terminar
    }
}
```

**Características**:
- O(1) allocation
- Thread-safe (via atomics)
- Sem free() individual
- Sem fragmentação
- Perfeito para bootloader (lifetime curto)

---

## Paging x86_64

### Estrutura de 4 Níveis

```
Virtual Address (48 bits):
|47   39|38   30|29   21|20   12|11     0|
| PML4  |  PDPT |   PD  |   PT  | Offset |
  9 bits  9 bits  9 bits  9 bits  12 bits

CR3 aponta para PML4
```

---

### Page Table Entry (64 bits)

```
Bits:
63      : NX (No Execute)
62-52   : Available (uso do OS)
51-12   : Physical Address (4KB aligned)
11-9    : Available
8       : Global
7       : PS (Page Size: 0=4KB, 1=2MB/1GB)
6       : Dirty
5       : Accessed
4       : Cache Disable
3       : Write-Through
2       : User/Supervisor
1       : Read/Write
0       : Present
```

---

### PageTableManager

```rust
pub struct PageTableManager {
    pml4_phys_addr: u64,
}

impl PageTableManager {
    pub fn new(allocator: &mut impl FrameAllocator) -> Result<Self> {
        // Alocar PML4
        let pml4 = allocator.allocate_frame(1)?;
        
        // Zerar
        unsafe {
            core::ptr::write_bytes(pml4 as *mut u8, 0, 4096);
        }
        
        Ok(Self { pml4_phys_addr: pml4 })
    }
    
    pub fn pml4_addr(&self) -> u64 {
        self.pml4_phys_addr
    }
    
    pub fn map_kernel(
        &mut self,
        phys: u64,
        virt: u64,
        pages: usize,
        allocator: &mut impl FrameAllocator,
    ) -> Result<()> {
        // Para cada página:
        for i in 0..pages {
            let phys_addr = phys + (i as u64 * 4096);
            let virt_addr = virt + (i as u64 * 4096);
            
            // Extrair índices
            let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
            let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
            let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
            let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
            
            // Walk page tables (criar se necessário)
            let pml4 = unsafe { &mut *(self.pml4_phys_addr as *mut PageTable) };
            let pdpt = self.get_or_create_table(&mut pml4[pml4_idx], allocator)?;
            let pd = self.get_or_create_table(&mut pdpt[pdpt_idx], allocator)?;
            let pt = self.get_or_create_table(&mut pd[pd_idx], allocator)?;
            
            // Mapear página
            pt[pt_idx] = PageTableEntry::new(phys_addr, PAGE_PRESENT | PAGE_WRITABLE);
        }
        
        Ok(())
    }
}
```

---

### Huge Pages (2 MB)

Para reduzir TLB misses, usar páginas de 2 MB:

```rust
// Set PS bit na PD entry
let pd_entry = PageTableEntry::new(
    phys_addr, 
    PAGE_PRESENT | PAGE_WRITABLE | PAGE_SIZE
);
pd[pd_idx] = pd_entry;

// Pula PT (não precisa)
```

**Vantagem**: Menos níveis de tabela, melhor performance  
**Limitação**: Alinhamento de 2 MB necessário

---

### NX Bit (No-Execute)

Proteger páginas de dados contra execução:

```rust
const PAGE_NO_EXEC: u64 = 1 << 63;

// Página de código (executável)
pt[idx] = PageTableEntry::new(code_addr, PAGE_PRESENT);

// Página de dados (não-executável)
pt[idx] = PageTableEntry::new(data_addr, PAGE_PRESENT | PAGE_WRITABLE | PAGE_NO_EXEC);
```

---

## Memory Map Handoff

### UEFI Memory Map

Obtido via `GetMemoryMap()`:

```rust
#[repr(C)]
pub struct UefiMemoryDescriptor {
    pub type_: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}
```

**Tipos**:
- `EfiLoaderCode`: Código do bootloader
- `EfiLoaderData`: Dados do bootloader
- `EfiBootServicesCode/Data`: Firmware UEFI (pode ser reutilizado)
- `EfiRuntimeServicesCode/Data`: Firmware UEFI (não pode reutilizar)
- `EfiConventionalMemory`: RAM livre
- `EfiACPIReclaimMemory`: ACPI tables (pode reclamar depois de ler)
- `EfiACPIMemoryNVS`: ACPI NVS (não pode reclamar)

---

### Conversão para Formato do Kernel

```rust
pub enum MemoryType {
    Usable,                  // RAM livre
    Reserved,                // Não tocar
    AcpiReclaimable,         // Pode reclamar após ler ACPI
    AcpiNvs,                 // Não pode reclamar
    BadMemory,               // RAM defeituosa
    BootloaderReclaimable,   // Usado pelo bootloader (pode reclamar)
    KernelAndModules,        // Kernel e módulos
    Framebuffer,             // VRAM
}

pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub type_: MemoryType,
}

fn convert_uefi_map(uefi_map: &[UefiMemoryDescriptor]) -> Vec<MemoryMapEntry> {
    let mut kernel_map = Vec::new();
    
    for desc in uefi_map {
        let type_ = match desc.type_ {
            EFI_CONVENTIONAL_MEMORY => MemoryType::Usable,
            EFI_LOADER_CODE | EFI_LOADER_DATA => MemoryType::BootloaderReclaimable,
            EFI_BOOT_SERVICES_CODE | EFI_BOOT_SERVICES_DATA => MemoryType::Usable,
            EFI_RUNTIME_SERVICES_CODE | EFI_RUNTIME_SERVICES_DATA => MemoryType::Reserved,
            EFI_ACPI_RECLAIM_MEMORY => MemoryType::AcpiReclaimable,
            EFI_ACPI_MEMORY_NVS => MemoryType::AcpiNvs,
            _ => MemoryType::Reserved,
        };
        
        kernel_map.push(MemoryMapEntry {
            base: desc.physical_start,
            length: desc.number_of_pages * 4096,
            type_,
        });
    }
    
    kernel_map
}
```

---

### Regiões Críticas

O kernel DEVE preservar:
- ACPI tables (até ler e copiar)
- Framebuffer
- Runtime Services código/dados (se usar UEFI runtime)

O kernel PODE reutilizar:
- BootloaderReclaimable (após copiar dados necessários)
- AcpiReclaimable (após parsear ACPI)

---

## Performance

### TLB (Translation Lookaside Buffer)

Cache de traduções virtuais → físicas.

**Otimizações**:
- Usar huge pages (2 MB / 1 GB)
- Minimizar mudanças de CR3
- Usar PCID (Process Context ID) para evitar flush completo

---

### Alinhamento

Sempre alinhar alocações:
- Páginas: 4 KB (0x1000)
- Huge pages: 2 MB (0x200000)
- Page tables: 4 KB

```rust
fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}
```

---

## Debugging

### Dumpar Page Tables

```rust
unsafe fn dump_page_table(pml4_addr: u64) {
    let pml4 = &*(pml4_addr as *const PageTable);
    
    for (i, entry) in pml4.iter().enumerate() {
        if entry.is_present() {
            println!("PML4[{}]: {:#x}", i, entry.addr());
        }
    }
}
```

### Verificar Mapeamento

```rust
fn is_mapped(pml4: u64, virt: u64) -> bool {
    // Walk page tables
    // Retornar true se presente
}
```

---

**Última Atualização**: 2025-12-21
