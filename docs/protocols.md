# Ignite Boot Protocols Documentation

This document provides comprehensive information about the 5 boot protocols supported by Ignite bootloader v0.4.0.

## Overview

Ignite implements a flexible protocol abstraction system that allows it to boot different types of kernels using the appropriate boot protocol. Each protocol implements the `BootProtocol` trait defined in `src/protos/mod.rs`.

## Protocol Abstraction

### BootProtocol Trait

```rust
pub trait BootProtocol {
    /// Validate that the kernel is compatible with this protocol
    fn validate(&self, kernel: &[u8]) -> Result<()>;
   
    /// Prepare boot (load kernel, setup structures)
    fn prepare(&mut self, kernel: &[u8], cmdline: Option<&str>, modules: &[LoadedFile]) -> Result<BootInfo>;
    
    /// Get the entry point address
    fn entry_point(&self) -> u64;
    
    /// Get protocol name
    fn name(&self) -> &'static str;
}
```

### BootInfo Structure

Contains all information needed to jump to the kernel:

```rust
pub struct BootInfo {
    pub entry_point: u64,       // Where to jump
    pub kernel_base: u64,       // Kernel load address
    pub kernel_size: u64,       // Size in bytes
    pub stack_ptr: Option<u64>, // Stack pointer (if needed)
    pub boot_info_ptr: u64,     // Protocol-specific info structure
    pub registers: ProtocolRegisters, // CPU register values
}
```

---

## 1. Limine Protocol

**File:** `src/protos/limine.rs`  
**Used For:** Redstone OS (native protocol)  
**Status:** ✅ Fully Implemented

### Description

The Limine protocol is the native boot protocol for Redstone OS. It's a simple ELF-based protocol that uses the existing `ElfLoader` to load the kernel and passes minimal boot information.

### Implementation

- Uses `ElfLoader` to parse and load ELF64 kernels
- Minimal boot info structure
- Direct entry point jump
- No complex boot structures needed

### Boot Information

Currently minimal. Can be extended with Limine-specific requests/responses in the future.

### Usage in Config

```ini
/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs
```

---

## 2. Linux Boot Protocol

**File:** `src/protos/linux.rs`  
**Used For:** Linux kernels (bzImage format)  
**Status:** 🚧 Mostly Complete (boot_params TODO)

### Description

Implements the Linux x86/x86_64 boot protocol for loading bzImage kernels, initrd, and passing kernel command lines.

### Key Features

- **SetupHeader Parsing** - Reads Linux kernel setup header at offset 0x1F1
- **Magic Validation** - Verifies "HdrS" (0x53726448) and boot flag (0xAA55)
- **Relocatable Kernel Support** - Respects `pref_address` for relocatable kernels
- **Command Line** - Allocates and passes kernel command line
- **Initrd Support** - Loads initrd/initramfs from modules

### Setup Header Structure

The protocol parses a comprehensive setup header including:
- Boot protocol version
- Kernel version  
- Relocatable kernel flag
- Preferred load address
- Kernel alignment requirements
- Command line size limit

### Memory Layout

1. **Setup Sectors** - First part of bzImage (512 bytes each)
2. **Protected Mode Kernel** - Starts after setup
3. **Command Line** - Allocated separately
4. **Initrd** - Loaded from first module

### TODO

- Complete `boot_params` structure with:
  - E820 memory map
  - Framebuffer info
  - ACPI tables
  - EFI information

### Usage in Config

```ini
/Linux
    protocol: linux
    kernel_path: boot():/vmlinuz
    module_path: boot():/initrd.img
    cmdline: root=/dev/sda1 quiet splash
```

### References

- [Linux x86 Boot Protocol](https://www.kernel.org/doc/html/latest/x86/boot.html)

---

## 3. Multiboot 1 Protocol

**File:** `src/protos/multiboot1.rs`  
**Used For:** GRUB legacy, FreeBSD, older systems  
**Status:** 🚧 Mostly Complete (MBI creation TODO)

### Description

Implements the classic Multiboot 1 specification. Supports both "a.out kludge" and ELF format kernels.

### Key Features

- **Header Search** - Searches first 8KB for Multiboot header (4-byte aligned)
- **Checksum Validation** - Verifies `magic + flags + checksum == 0`
- **A.out Kludge** - Loads at specific addresses from header
- **ELF Support** - Uses `ElfLoader` for ELF kernels
- **MBI Creation** - Creates Multiboot Info structure

### Multiboot Header

```c
struct multiboot_header {
    uint32_t magic;       // 0x1BADB002
    uint32_t flags;
    uint32_t checksum;    // -(magic + flags)
    
    // If flags[16] (a.out kludge):
    uint32_t header_addr;
    uint32_t load_addr;
    uint32_t load_end_addr;
    uint32_t bss_end_addr;
    uint32_t entry_addr;
    
    // If flags[2] (video mode):
    uint32_t mode_type;
    uint32_t width;
    uint32_t height;
    uint32_t depth;
};
```

### Boot Information

Passes Multiboot Info (MBI) structure pointer in EBX register with magic value 0x2BADB002 in EAX.

### TODO

- Complete MBI creation with:
  - Memory map (E820-style)
  - Modules information
  - Boot loader name
  - Framebuffer info

### Usage in Config

```ini
/Multiboot Kernel
    protocol: multiboot1
    kernel_path: boot():/kernel.mb1
    module_path: boot():/module1
    module_path: boot():/module2
```

### References

- [Multiboot 1 Specification](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html)

---

## 4. Multiboot 2 Protocol

**File:** `src/protos/multiboot2.rs`  
**Used For:** Modern multiboot systems  
**Status:** 🚧 Structure Complete (tag creation TODO)

### Description

Implements Multiboot 2 specification with modern tag-based boot information structure.

### Key Features

- **Tag-Based Header** - More flexible than Multiboot 1  
- **ELF Loading** - Uses `ElfLoader` exclusively
- **Extensible Info Structure** - Tag-based MBI2
- **Alignment Requirements** - Proper 8-byte alignment

### Multiboot2 Header

```c
struct multiboot2_header {
    uint32_t magic;           // 0xE85250D6
    uint32_t architecture;     // 0 (i386), 4 (MIPS)
    uint32_t header_length;
    uint32_t checksum;        // -(magic + arch + length)
    
    // Followed by tags
};
```

### Boot Information Tags

The protocol should create MBI2 with tags for:
- Basic memory info
- Boot command line
- Modules
- Memory map
- Framebuffer
- ELF symbols
- ACPI tables

### TODO

- Implement complete tag creation
- Add all required/optional tags
- Proper alignment handling

### Usage in Config

```ini
/Multiboot2 Kernel
    protocol: multiboot2
    kernel_path: boot():/kernel.mb2
    cmdline: boot_option=value
```

### References

- [Multiboot 2 Specification](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html)

---

## 5. EFI Chainload Protocol

**File:** `src/protos/chainload.rs`  
**Used For:** Loading other UEFI applications/bootloaders  
**Status:** ✅ Basic Implementation

### Description

Chainloads other EFI applications like GRUB, Windows Boot Manager, or other bootloaders.

### Key Features

- **PE/COFF Validation** - Validates PE headers
- **EFI Application Loading** - Loads .efi files
- **BIOS Chainload Placeholder** - Structure for future BIOS support

### PE/COFF Validation

Checks for valid:
- DOS header (MZ signature)
- PE header (PE\\0\\0 signature)
- Optional header with EFI subsystem

### Usage in Config

```ini
/Windows Boot Manager
    protocol: efi
    kernel_path: boot():/EFI/Microsoft/Boot/bootmgfw.efi

/GRUB
    protocol: efi
    kernel_path: boot():/EFI/grub/grubx64.efi
```

### Future: BIOS Chainload

Planned support for chainloading BIOS bootloaders from MBR/VBR.

---

## Protocol Selection

### Automatic Detection

The bootloader can auto-detect protocols by:

1. Checking for Multiboot magic in first 8KB
2. Checking for Linux setup header at 0x1F1
3. Checking for PE/COFF headers
4. Default to Limine/ELF for Redstone

### Manual Selection

Users specify protocol explicitly in config:

```ini
protocol: limine | linux | multiboot1 | multiboot2 | efi
```

---

## Register Conventions

### x86-64 Register Setup

**Limine:**
- RIP: entry_point
- No special registers

**Linux:**
- RSI: boot_params pointer
- RIP: entry_point

**Multiboot 1:**
- EAX: 0x2BADB002 (magic)
- EBX: MBI pointer
- RIP: entry_point

**Multiboot 2:**
- EAX: 0x36D76289 (magic)
- EBX: MBI2 pointer  
- RIP: entry_point

**EFI Chainload:**
- Calls LoadImage/StartImage

---

## Adding New Protocols

To add a new boot protocol:

1. Create new file in `src/protos/`
2. Implement `BootProtocol` trait
3. Add protocol to `src/protos/mod.rs`
4. Update config parser to recognize protocol name
5. Document protocol here

---

**Last Updated:** 2025-12-18  
**Version:** 0.4.0
