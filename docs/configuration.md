# Ignite Configuration Guide

Complete guide to configuring the Ignite bootloader using `ignite.conf` configuration files.

## Configuration File Format

Ignite uses a Limine-compatible configuration format that supports:
- Global options
- Boot menu entries
- Hierarchical sub-entries
- Macro expansion
- Advanced path resolution

## File Location

Configuration file should be placed at:
- `boot():/ignite.conf` (recommended)
- `boot():/boot.cfg` (alternative)
- Root of boot partition

## Basic Configuration

### Minimal Example

```ini
timeout: 5
default_entry: 1

/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
```

This creates

 a single boot entry with 5-second timeout.

### Complete Example

```ini
# Global Configuration
timeout: 10
default_entry: 1
quiet: no
serial: yes
verbose: yes

interface_resolution: 1920x1080
interface_branding: Redstone OS Bootloader v0.4
wallpaper: boot():/boot/bg.png
wallpaper_style: centered

editor_enabled: yes

# Custom Macros
${OS_NAME}=Redstone
${VERSION}=0.1.0

# Boot Entries
/Redstone OS ${VERSION}
    comment: Default boot entry for Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs
    cmdline: quiet splash loglevel=3

//Advanced Options
    comment: Boot with verbose logging
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs
    cmdline: verbose debug loglevel=7

/Linux
    comment: Linux Kernel 6.1
    protocol: linux
    kernel_path: boot():/vmlinuz-6.1
    module_path: boot():/initrd.img-6.1
    cmdline: root=/dev/sda2 quiet splash

/Recovery
    comment: Recovery shell
    protocol: limine
    kernel_path: boot():/forge-recovery
    module_path: boot():/initfs-recovery
    cmdline: recovery single
```

## Global Options

### Basic Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `timeout` | int | 5 | Seconds before auto-boot (0=instant, "no"=wait forever) |
| `default_entry` | int | 1 | Default entry index (1-based) |
| `quiet` | yes/no | no | Suppress boot messages |
| `serial` | yes/no | no | Enable serial output |
| `serial_baudrate` | int | 115200 | Serial port baud rate |
| `verbose` | yes/no | no | Verbose boot messages |

### Interface Options

| Option | Type | Description |
|--------|------|-------------|
| `interface_resolution` | WxH | Screen resolution (e.g., "1920x1080") |
| `interface_branding` | string | Branding text shown in menu |
| `wallpaper` | path | Background image path |
| `wallpaper_style` | style | `centered`, `stretched`, `tiled` |
| `editor_enabled` | yes/no | Enable config editor (E key) |

## Boot Entries

### Entry Hierarchy

Entries are defined with `/` prefix. Sub-entries use `//`:

```ini
/Main Entry
    ...options...

//Sub Entry 1
    ...options...

//Sub Entry 2
    ...options...

/Another Main Entry
    ...options...
```

### Expanded Entries

Add `+` after slashes to auto-expand sub-entries:

```ini
/+Redstone OS
    ...
//Option 1
    ...
//Option 2
    ...
```

### Entry Options

**Required:**
- `protocol:` - Boot protocol (limine, linux, multiboot1, multiboot2, efi)
- `kernel_path:` - Path to kernel file

**Optional:**
- `comment:` - Description shown in menu
- `cmdline:` - Kernel command line arguments
- `module_path:` - Module/initrd path (can specify multiple)
- `module_string:` - Command line for module
- `resolution:` - Entry-specific resolution (WxHxBPP)
- `textmode:` - Force text mode (yes/no)
- `dtb_path:` - Device tree blob path (ARM/RISC-V)
- `kaslr:` - Enable KASLR (yes/no)

## Path System

Advanced path resolution with multiple formats:

### Boot Partition Paths

```ini
boot():/kernel          # Default boot partition
boot(2):/kernel         # Boot partition index 2
```

### Hard Disk Paths

```ini
hdd(0:1):/kernel        # Disk 0, Partition 1
hdd(1:2):/kernel        # Disk 1, Partition 2
```

### GUID/UUID Paths

```ini
guid(550e8400-e29b-41d4-a716-446655440000):/kernel
uuid(550e8400-e29b-41d4-a716-446655440000):/kernel
```

### Filesystem Label Paths

```ini
fslabel(BOOT):/kernel
fslabel(REDSTONE):/kernel
```

### Hash Verification

Add `#hash` suffix to verify file integrity:

```ini
kernel_path: boot():/forge#1a2b3c4d5e6f...
```

Uses BLAKE2B hashing.

## Macro System

### Built-in Macros

| Macro | Example Value | Description |
|-------|---------------|-------------|
| `${ARCH}` | x86-64 | CPU architecture |
| `${FW_TYPE}` | UEFI | Firmware type |

### Custom Macros

Define custom macros:

```ini
${OS_NAME}=Redstone
${VERSION}=1.0
${ROOT_DEV}=/dev/nvme0n1p2

/My ${OS_NAME} v${VERSION}
    cmdline: root=${ROOT_DEV}
```

Expands to:
```ini
/My Redstone v1.0
    cmdline: root=/dev/nvme0n1p2
```

## Protocol-Specific Configuration

### Limine Protocol

```ini
/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs
    resolution: 1920x1080x32
```

### Linux Protocol

```ini
/Linux
    protocol: linux
    kernel_path: boot():/vmlinuz
    module_path: boot():/initrd.img  # This is the initrd
    cmdline: root=/dev/sda1 quiet splash
```

### Multiboot 1

```ini
/FreeBSD
    protocol: multiboot1
    kernel_path: boot():/kernel.mb
    module_path: boot():/module1
    module_string: module cmdline
    module_path: boot():/module2
```

### Multiboot 2

```ini
/GRUB Kernel
    protocol: multiboot2
    kernel_path: boot():/kernel.mb2
    cmdline: boot_option=value
```

### EFI Chainload

```ini
/Windows
    protocol: efi
    kernel_path: boot():/EFI/Microsoft/Boot/bootmgfw.efi

/GRUB
    protocol: efi
    kernel_path: boot():/EFI/grub/grubx64.efi
```

## Validation

The config parser validates:

- **Syntax** - Proper key:value format
- **Required fields** - Protocol and kernel_path must be present
- **Protocol names** - Must be valid (limine, linux, multiboot1, multiboot2, efi)
- **Default entry** - Must be in range 1..N
- **Entry count** - At least one entry required

Errors are displayed with line numbers before falling back to defaults.

## Examples

### Multi-Boot System

```ini
timeout: 5
default_entry: 1

/Redstone OS
    protocol: limine
    kernel_path: boot():/forge
    module_path: boot():/initfs

/Arch Linux
    protocol: linux
    kernel_path: hdd(0:2):/vmlinuz-linux
    module_path: hdd(0:2):/initramfs-linux.img
    cmdline: root=/dev/nvme0n1p2 rw

/Windows 11
    protocol: efi
    kernel_path: guid(28FB-A886):/EFI/Microsoft/Boot/bootmgfw.efi

/GRUB Rescue
    protocol: efi
    kernel_path: boot():/EFI/grub/grubx64.efi
```

### Development System

```ini
timeout: 10
verbose: yes
serial: yes
editor_enabled: yes

${KERNEL_DIR}=boot():/kernels

/Stable Kernel
    protocol: limine
    kernel_path: ${KERNEL_DIR}/forge-stable
    module_path: boot():/initfs

/Development Kernel (Debug)
    protocol: limine
    kernel_path: ${KERNEL_DIR}/forge-dev
    cmdline: debug loglevel=7 kgdb
```

### Secure Boot

```ini
${HASH_KERNEL}=1a2b3c4d5e6f7890abcdef...
${HASH_INITFS}=fedcba0987654321...

/Redstone OS (Verified)
    protocol: limine
    kernel_path: boot():/forge#${HASH_KERNEL}
    module_path: boot():/initfs#${HASH_INITFS}
```

## Tips & Best Practices

1. **Use Comments** - Document your entries with `comment:` for clarity
2. **Use Macros** - Reduce repetition with custom macros
3. **Hash Verification** - Use `#hash` for production systems
4. **Fallback Entries** - Always have a recovery/safe-mode entry
5. **Test Changes** - Use editor (E key) to test before permanent changes
6. **Serial Output** - Enable for debugging: `serial: yes`

## Troubleshooting

### Config Not Loading

1. Check file is at `boot():/ignite.conf`
2. Verify proper formatting (key: value)
3. Check for syntax errors
4. Enable verbose mode: `verbose: yes`

### Entry Not Appearing

1. Verify entry starts with `/`
2. Check for duplicate names
3. Ensure protocol is specified
4. Verify kernel_path is present

### Kernel Won't Boot

1. Verify protocol matches kernel type
2. Check kernel_path is correct
3. Verify paths are accessible
4. Check hash if using verification

---

**Last Updated:** 2025-12-18  
**Version:** 0.4.0
