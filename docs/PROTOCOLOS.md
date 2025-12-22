# Protocolos de Boot - Ignite Bootloader

Documentação técnica dos protocolos de boot suportados.

## 📋 Protocolos Suportados

- [Redstone/Limine](#redstonelimine-protocolo-nativo)
- [Linux Boot Protocol](#linux-boot-protocol)
- [Multiboot2](#multiboot2)
- [UEFI Chainload](#uefi-chainload)

---

## Redstone/Limine (Protocolo Nativo)

### Especificação

Protocolo nativo otimizado para kernels modernos, inspirado no [Limine Boot Protocol](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md).

**Formato**: ELF64  
**Magic Bytes**: `0x7F 0x45 0x4C 0x46` (ELF header)

---

### Estrutura BootInfo

Passada via RDI:

```rust
#[repr(C)]
pub struct BootInfo {
    pub magic: u64,                      // 0x524544_53544F4E45 ("REDSTONE")
    pub version: u32,                    // Versão do protocolo (ex: 1)
    
    // Framebuffer
    pub framebuffer: FramebufferInfo,
    
    // Memória
    pub memory_map_addr: u64,
    pub memory_map_entries: usize,
    
    // ACPI
    pub rsdp_addr: u64,                  // ACPI RSDP physical address
    
    // Kernel
    pub kernel_cmdline: *const u8,       // Null-terminated UTF-8
    pub kernel_physical_base: u64,
    pub kernel_virtual_base: u64,
    
    // Módulos (initrd, drivers)
    pub modules: &'static [Module],
    
    // Bootloader info
    pub bootloader_name: *const u8,      // "Ignite"
    pub bootloader_version: *const u8,   // "0.1.0"
}

#[repr(C)]
pub struct Module {
    pub address: u64,
    pub size: usize,
    pub cmdline: *const u8,
}

#[repr(C)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,  // PixelFormat enum
}
```

---

### Entry Point

**Calling Convention**: System V AMD64
- RDI: `&BootInfo`
- Interrupções: Desabilitadas (CLI)
- Paging: Habilitado (CR3 apontando para page tables do kernel)

```asm
; Kernel entry point
global _start
_start:
    ; RDI já contém ponteiro para BootInfo
    call kernel_main
    ; kernel_main não deve retornar
    cli
    hlt
```

```rust
// src/main.rs (kernel)
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    kernel_main(boot_info);
}
```

---

### Memory Map

Formato:

```rust
#[repr(C)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub type_: MemoryType,
}

#[repr(u32)]
pub enum MemoryType {
    Usable = 0,           // RAM livre
    Reserved = 1,         // Reservado (firmware)
    AcpiReclaimable = 2,  // Pode ser reutilizado após ler ACPI
    AcpiNvs = 3,          // ACPI NVS (não pode reclamar)
    BadMemory = 4,        // RAM defeituosa
    BootloaderReclaimable = 5,  // Usado pelo bootloader (pode reclamar)
    KernelAndModules = 6, // Kernel e módulos
    Framebuffer = 7,      // VRAM
}
```

---

### Paging

**Layout de Memória Virtual**:

```
0x0000000000000000 - 0x0000007FFFFFFFFF: Userspace (128 TB)
0xFFFF800000000000 - 0xFFFF87FFFFFFFFFF: Direct map de toda RAM física (128 TB)
0xFFFFFFFF80000000 - 0xFFFFFFFFFFFFFFFF: Kernel (2 GB)
```

**Mapeamentos criados pelo bootloader**:
- Kernel code/data em `0xFFFFFFFF80000000` (higher-half)
- Direct map de toda RAM em `0xFFFF800000000000`
- Framebuffer identity-mapped

---

### Exemplo de Uso

```rust
#[no_mangle]
pub extern "C" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Validar magic
    assert_eq!(boot_info.magic, 0x524544_53544F4E45);
    
    // Acessar framebuffer
    let fb = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.framebuffer.address as *mut u32,
            boot_info.framebuffer.size / 4,
        )
    };
    
    // Desenhar pixel
    fb[0] = 0xFF0000; // Vermelho
    
    // Ler memory map
    let mem_map = unsafe {
        core::slice::from_raw_parts(
            boot_info.memory_map_addr as *const MemoryMapEntry,
            boot_info.memory_map_entries,
        )
    };
    
    for entry in mem_map {
        println!("Memory: {:#x} - {:#x} ({:?})", 
            entry.base, entry.base + entry.length, entry.type_);
    }
    
    // Inicializar kernel...
    loop {}
}
```

---

## Linux Boot Protocol

### Especificação

Baseado no [Linux x86 Boot Protocol](https://www.kernel.org/doc/html/latest/x86/boot.html).

**Formato**: bzImage (compressed kernel image)  
**Magic**: `0x53726448` ("HdrS") no setup header (offset 0x202)

---

### Estrutura boot_params

Passada via RSI:

```rust
#[repr(C)]
pub struct BootParams {
    pub screen_info: ScreenInfo,         // 0x000
    pub apm_bios_info: ApmBiosInfo,      // 0x040
   pub _pad2: [u8; 4],                   // 0x054
    pub tboot_addr: u64,                 // 0x058
    pub ist_info: IstInfo,               // 0x060
    pub _pad3: [u8; 16],                 // 0x070
    pub hd0_info: [u8; 16],              // 0x080 (obsoleto)
    pub hd1_info: [u8; 16],              // 0x090 (obsoleto)
    pub sys_desc_table: SysDescTable,    // 0x0a0 (obsoleto)
    pub olpc_ofw_header: OlpcOfwHeader,  // 0x0b0
    pub ext_ramdisk_image: u32,          // 0x0c0
    pub ext_ramdisk_size: u32,           // 0x0c4
    pub ext_cmd_line_ptr: u32,           // 0x0c8
    pub _pad4: [u8; 116],                // 0x0cc
    pub edid_info: EdidInfo,             // 0x140
    pub efi_info: EfiInfo,               // 0x1c0
    pub alt_mem_k: u32,                  // 0x1e0
    pub scratch: u32,                    // 0x1e4
    pub e820_entries: u8,                // 0x1e8
    pub eddbuf_entries: u8,              // 0x1e9
    pub edd_mbr_sig_buf_entries: u8,     // 0x1ea
    pub kbd_status: u8,                  // 0x1eb
    pub secure_boot: u8,                 // 0x1ec
    pub _pad5 [u8; 2],                   // 0x1ed
    pub sentinel: u8,                    // 0x1ef
    pub _pad6: [u8; 1],                  // 0x1f0
    pub hdr: SetupHeader,                // 0x1f1
    pub _pad7: [u8; 0x290 - 0x1f1 - size_of::<SetupHeader>()],
    pub edd_mbr_sig_buffer: [u32; 16],   // 0x290
    pub e820_table: [E820Entry; 128],    // 0x2d0
    pub _pad8: [u8; 48],                 // 0xcd0
    pub eddbuf: [EddInfo; 6],            // 0xd00
    pub _pad9: [u8; 276],                // 0xeec
}

#[repr(C)]
pub struct SetupHeader {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_dev: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,                     // "HdrS"
    pub version: u16,                    // Protocol version
    pub realmode_swtch: u32,
    pub start_sys_seg: u16,
    pub kernel_version: u16,
    pub type_of_loader: u8,
    pub loadflags: u8,
    pub setup_move_size: u16,
    pub code32_start: u32,
    pub ramdisk_image: u32,
    pub ramdisk_size: u32,
    pub bootsect_kludge: u32,
    pub heap_end_ptr: u16,
    pub ext_loader_ver: u8,
    pub ext_loader_type: u8,
    pub cmd_line_ptr: u32,
    pub initrd_addr_max: u32,
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub min_alignment: u8,
    pub xloadflags: u16,
    pub cmdline_size: u32,
    pub hardware_subarch: u32,
    pub hardware_subarch_data: u64,
    pub payload_offset: u32,
    pub payload_length: u32,
    pub setup_data: u64,
    pub pref_address: u64,
    pub init_size: u32,
    pub handover_offset: u32,
    pub kernel_info_offset: u32,
}
```

---

### Carregamento

1. **Parse Setup Header** (offset 0x1f1)
2. **Alocar boot_params** (uma página, zerada)
3. **Copiar setup header** para boot_params.hdr
4. **Configurar campos**:
   - `cmd_line_ptr`: Ponteiro para command line
   - `ramdisk_image`: Endereço físico do initrd
   - `ramdisk_size`: Tamanho do initrd
   - `type_of_loader`: 0xFF (tipo desconhecido)
   - `loadflags |= 0x01` (CAN_USE_HEAP)
5. **Carregar kernel** em memória protegida
6. **Carregar initrd** (se presente)
7. **Jump** para entry point

---

### Entry Point

```asm
; RAX = Não especificado
; RBX = Não especificado
; RCX = Não especificado
; RDX = Não especificado
; RSI = boot_params physical address
; RDI = Não especificado
; RBP = Não especificado
; RSP = Kernel deve configurar

; Interrupções: Desabilitadas
; Paging: Desabilitado (ou identity-mapped)
```

---

## Multiboot2

### Especificação

Baseado no [Multiboot2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html).

**Magic**: `0xE85250D6` no header Multiboot2  
**Formato**: ELF ou binário bruto com header

---

### Multiboot2 Header

Deve estar nos primeiros 32 KB do kernel:

```c
struct multiboot2_header {
    u32 magic;              // 0xE85250D6
    u32 architecture;       // 0 = i386, 4 = MIPS
    u32 header_length;
    u32 checksum;           // -(magic + architecture + header_length)
    // Tags seguem...
};
```

---

### MBI (Multiboot Information)

Passada via RBX:

```rust
#[repr(C)]
pub struct MultibootInfo {
    pub total_size: u32,
    pub reserved: u32,
    // Tags seguem...
}
```

**Tags comuns**:
- Boot command line
- Bootloader name
- Modules
- Basic memory info
- Memory map
- Framebuffer info
- ELF sections
- ACPI old/new RSDP

---

### Entry Point

```asm
; EAX = 0x36D76289 (magic)
; EBX = MBI physical address
; Outros registradores: Indefinidos
; Interrupções: Desabilitadas
; Paging: Desabilitado
```

---

## UEFI Chainload

### Especificação

Carrega e executa outro aplicativo UEFI usando `LoadImage()` e `StartImage()`.

**Formato**: PE32+ (Portable Executable)  
**Magic**: `MZ` (DOS header)

---

### Funcionamento

```rust
// 1. Carregar binário do disco
let efi_app_data = fs::read("shellx64.efi")?;

// 2. LoadImage (UEFI aloca memória e valida assinatura se Secure Boot ativo)
let mut child_handle = Handle::null();
let status = boot_services.load_image(
    false,                              // Boot policy
    image_handle,                       // Parent image
    null(),                             // Device path
    efi_app_data.as_ptr(),              // Source buffer
    efi_app_data.len(),
    &mut child_handle,
);

// 3. StartImage (transfere controle)
let status = boot_services.start_image(
    child_handle,
    &mut exit_data_size,
    &mut exit_data,
);

// 4. Se retornar (ex: usuário digitou 'exit' no shell)
// Ignite pode:
// - Voltar ao menu
// - Reiniciar sistema
// - Permitir outro chainload
```

---

### Casos de Uso

- **UEFI Shell**: Ferramenta de diagnóstico
- **Outro bootloader**: Ex: GRUB, Windows Boot Manager
- **Ferramentas UEFI**: Firmware update, memory test

---

## Resumo Comparativo

| Feature | Redstone/Limine | Linux | Multiboot2 | Chainload |
|---------|----------------|-------|------------|-----------|
| Formato | ELF64 | bzImage | ELF/Raw | PE32+ |
| Paging | Sim (higher-half) | Não | Não | N/A |
| Framebuffer | ✅ Completo | ✅ Básico | ✅ Completo | N/A |
| Memory Map | ✅ Detalhado | ✅ E820 | ✅ Tags | N/A |
| Modules | ✅ | ✅ (initrd) | ✅ | N/A |
| ACPI | ✅ RSDP | ✅ | ✅ | N/A |
| Entry ABI | System V | Linux-specific | Multiboot2 | UEFI |

---

**Última Atualização**: 2025-12-21
