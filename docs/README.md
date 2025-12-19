# Ignite Documentation

This directory contains detailed documentation for the Ignite UEFI bootloader.

## Documentation Files

### Core Documentation (Root Directory)

Located in the root `ignite/` directory:

- **[README.md](../README.md)** - Main project documentation with overview, features, and quick start
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history and release notes
- **[SECURITY.md](../SECURITY.md)** - Security policy and vulnerability reporting
- **[INDICE.md](../INDICE.md)** - Complete project index and file structure
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Contribution guidelines
- **[CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)** - Community code of conduct
- **[AUTHORS.md](../AUTHORS.md)** - List of contributors

### Detailed Documentation (This Directory)

- **[protocols.md](protocols.md)** - Complete guide to all 5 boot protocols
- **[configuration.md](configuration.md)** - Configuration file format and options
- **[development.md](development.md)** - Developer guide and setup instructions

## Quick Links

### For Users

- **Getting Started**: See [README.md](../README.md)
- **Configuration**: See [configuration.md](configuration.md)
- **Troubleshooting**: See [README.md](../README.md#troubleshooting)

### For Developers

- **Development Setup**: See [development.md](development.md#setup--prerequisites)
- **Architecture**: See [INDICE.md](../INDICE.md)
- **Adding Features**: See [development.md](development.md#adding-new-features)
- **Code Style**: See [development.md](development.md#coding-conventions)

### Protocol-Specific

- **Limine Protocol**: See [protocols.md](protocols.md#1-limine-protocol)
- **Linux Protocol**: See [protocols.md](protocols.md#2-linux-boot-protocol)
- **Multiboot 1**: See [protocols.md](protocols.md#3-multiboot-1-protocol)
- **Multiboot 2**: See [protocols.md](protocols.md#4-multiboot-2-protocol)
- **EFI Chainload**: See [protocols.md](protocols.md#5-efi-chainload-protocol)

## External Resources

### UEFI & Firmware

- [UEFI Specification](https://uefi.org/specifications) - Official UEFI specs
- [UEFI.org](https://uefi.org/) - UEFI Forum main site
- [TianoCore EDK II](https://www.tianocore.org/) - Reference UEFI implementation

### Boot Protocols

- [Linux Boot Protocol](https://www.kernel.org/doc/html/latest/x86/boot.html) - Official Linux docs
- [Multiboot 1 Spec](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html) - GNU Multiboot 1
- [Multiboot 2 Spec](https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html) - GNU Multiboot 2
- [Limine Boot Protocol](https://github.com/limine-bootloader/limine) - Limine project

### Rust & Development

- [Rust UEFI](https://github.com/rust-osdev/uefi-rs) - UEFI bindings for Rust
- [Rust Embedded Book](https://rust-embedded.github.io/book/) - Embedded Rust guide
- [OSDev Wiki](https://wiki.osdev.org/) - OS development resources
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - Dark arts of unsafe Rust

### Filesystems

- [FAT Specification](https://www.wiki.osdev.org/FAT) - FAT filesystem details
- [ISO 9660](https://wiki.osdev.org/ISO_9660) - CD/DVD filesystem
- [EFI System Partition](https://en.wikipedia.org/wiki/EFI_system_partition) - ESP overview

### Hardware Standards

- [ACPI Specification](https://uefi.org/specifications) - ACPI tables and power management
- [Device Tree](https://www.devicetree.org/) - Device Tree specification
- [PCI Specification](https://pcisig.com/) - PCI bus standard

## Documentation Standards

All documentation follows these principles:

1. **Accuracy** - Information must be correct and up-to-date
2. **Completeness** - Cover all features and edge cases
3. **Clarity** - Write for the target audience
4. **Examples** - Provide practical examples
5. **Maintenance** - Update with code changes

## Contributing to Documentation

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

**Documentation-specific tips:**
- Use clear, concise language
- Include code examples where helpful
- Link to related documentation
- Update CHANGELOG.md when adding features
- Keep index files (INDICE.md) current

## Version Information

**Current Version:** 0.4.0  
**Last Documentation Update:** 2025-12-18  

See [CHANGELOG.md](../CHANGELOG.md) for detailed version history.

---

## Table of Contents

### Main README
- Overview
- Features
- Architecture
- Building
- Configuration
- Usage
- Roadmap

### Protocols (protocols.md)
- Protocol Abstraction
- Limine Protocol
- Linux Boot Protocol
- Multiboot 1
- Multiboot 2
- EFI Chainload
- Protocol Selection
- Register Conventions

### Configuration (configuration.md)
- File format
- Global options
- Boot entries
- Path system
- Macro system
- Protocol-specific config
- Validation
- Examples

### Development (development.md)
- Setup & Prerequisites
- Building
- Code Quality
- Testing
- Development Workflow
- Coding Conventions
- Adding Features
- Common Tasks
- Troubleshooting

---

For questions or clarifications, please:
1. Check existing documentation
2. Search GitHub issues
3. Create a new issue with the `documentation` label

**Happy developing with Ignite!** 🔥
