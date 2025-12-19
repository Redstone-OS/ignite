# Ignite Development Guide

Guide for developers working on the Ignite bootloader.

##Setup & Prerequisites

### Required Tools

- **Rust** - Nightly toolchain (edition 2024)
- **Target** - `x86_64-unknown-uefi`
- **QEMU** - For testing (optional but recommended)
- **OVMF** - UEFI firmware for QEMU

### Installation

```bash
# Install Rust nightly
rustup toolchain install nightly
rustup default nightly

# Add UEFI target
rustup target add x86_64-unknown-uefi

# Install developer tools
rustup component add clippy rustfmt

# Install QEMU (Ubuntu/Debian)
sudo apt install qemu-system-x86 ovmf
```

## Project Structure

See [INDICE.md](../INDICE.md) for complete file structure.

**Key directories:**
- `src/protos/` - Boot protocol implementations
- `src/config/` - Configuration system
- `src/ui/` - User interface
- `src/fs/` - Filesystem drivers
- `src/hardware/` - Hardware abstraction (ACPI, FDT)

## Building

### Debug Build

```bash
cargo build --target x86_64-unknown-uefi
```

Output: `target/x86_64-unknown-uefi/debug/ignite.efi`

### Release Build

```bash
cargo build --target x86_64-unknown-uefi --release
```

Output: `target/x86_64-unknown-uefi/release/ignite.efi`

### Quick Check

```bash
cargo check --target x86_64-unknown-uefi
```

Faster than full build, checks compilation errors only.

## Code Quality

### Formatting

```bash
# Check formatting
cargo fmt -- --check

# Auto-format
cargo fmt
```

Configuration is in `rustfmt.toml`.

### Linting

```bash
# Run clippy
cargo clippy --target x86_64-unknown-uefi

# Fix auto-fixable issues
cargo clippy --target x86_64-unknown-uefi --fix
```

Configuration is in `.clippy.toml`.

### Testing

```bash
# Run tests
cargo test --lib

# Run specific test
cargo test --lib test_name
```

## Testing in QEMU

### Basic QEMU Test

```bash
# Create ESP directory structure
mkdir -p esp/EFI/BOOT
cp target/x86_64-unknown-uefi/debug/ignite.efi esp/EFI/BOOT/BOOTX64.EFI
cp forge esp/
cp initfs esp/
cp ignite.conf esp/

# Run QEMU
qemu-system-x86_64 \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive format=raw,file=fat:rw:esp \
  -m 512M \
  -serial stdio
```

### QEMU with Debug

```bash
qemu-system-x86_64 \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive format=raw,file=fat:rw:esp \
  -m 512M \
  -serial stdio \
  -d int,cpu_reset \
  -no-reboot \
  -no-shutdown
```

### GDB Debugging

```bash
# Terminal 1: QEMU with GDB stub
qemu-system-x86_64 \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive format=raw,file=fat:rw:esp \
  -s -S

# Terminal 2: GDB
gdb target/x86_64-unknown-uefi/debug/ignite.efi
(gdb) target remote :1234
(gdb) break boot
(gdb) continue
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Edit code, following conventions below.

### 3. Test Locally

```bash
cargo build --target x86_64-unknown-uefi
cargo test --lib
cargo clippy --target x86_64-unknown-uefi
cargo fmt -- --check
```

### 4. Test in QEMU

Build and run in QEMU to verify functionality.

### 5. Commit

```bash
git add .
git commit -m "feat: add feature X

- Implementation detail 1
- Implementation detail 2

Closes #123"
```

### 6. Push & PR

```bash
git push origin feature/my-feature
```

Then create Pull Request on GitHub.

## Coding Conventions

### Style Guide

- **Formatting**: Follow `rustfmt.toml`
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types
- **Line length**: Max 100 characters
- **Imports**: Group by `std` → `external` → `internal`

### Documentation

All public items must have doc comments:

```rust
/// Loads a kernel using the specified protocol
///
/// # Arguments
///
/// * `kernel` - Kernel image bytes
/// * `protocol` - Boot protocol to use
///
/// # Returns
///
/// BootInfo structure on success
///
/// # Errors
///
/// Returns error if kernel is invalid or loading fails
pub fn load_kernel(kernel: &[u8], protocol: &dyn BootProtocol) -> Result<BootInfo> {
    // ...
}
```

### Error Handling

Use `Result<T>` and the `?` operator:

```rust
fn my_function() -> Result<()> {
    let data = load_file(path)?;  // Propagate errors
    process(data)?;
    Ok(())
}
```

Never use `.unwrap()` or `.expect()` except in tests.

### Unsafe Code

Minimize `unsafe` usage:

1. **Document why unsafe is necessary**
2. **Explain safety invariants**
3. **Keep unsafe blocks small**

```rust
// SAFETY: ptr is valid and aligned because...
unsafe {
    core::ptr::write(ptr, value);
}
```

### Module Organization

Each module should have:
- `mod.rs` - Public API and re-exports
- Implementation files - Internal details

```rust
// mod.rs
pub mod parser;
pub mod loader;

pub use parser::Parser;
pub use loader::Loader;
```

## Adding New Features

### Adding a New Boot Protocol

1. Create `src/protos/myprotocol.rs`
2. Implement `BootProtocol` trait
3. Add to `src/protos/mod.rs`
4. Update config parser
5. Add tests
6. Document in `docs/protocols.md`

### Adding Config Options

1. Add field to `BootConfig` in `src/config/types.rs`
2. Add parsing in `src/config/parser.rs`
3. Add validation in `src/config/validator.rs`
4. Update `docs/configuration.md`

### Adding Filesystem Driver

1. Create `src/fs/myfs.rs`
2. Implement mount and read functions
3. Add to `src/fs/mod.rs`
4. Add tests
5. Document

## Common Tasks

### Updating Dependencies

```bash
# Check for outdated dependencies
cargo outdated

# Update Cargo.lock
cargo update

# Update specific dependency
cargo update -p dependency-name
```

### Adding Dependencies

Edit `Cargo.toml`:

```toml
[dependencies]
my-crate = { version = "1.0", default-features = false }
```

**Important**: Use `default-features = false` for `no_std` compatibility.

### Debugging Compilation Errors

1. Run with full backtrace:
```bash
RUST_BACKTRACE=full cargo build --target x86_64-unknown-uefi
```

2. Check specific error:
```bash
rustc --explain E0XXX
```

3. Use `cargo expand` to see macro expansion:
```bash
cargo install cargo-expand
cargo expand --target x86_64-unknown-uefi
```

## Architecture-Specific Notes

### Memory Layout

UEFI uses:
- `BootServicesCode` - Bootloader code
- `BootServicesData` - Bootloader data
- `LoaderCode` - Kernel code
- `LoaderData` - Kernel data

After `exit_boot_services()`, only Loader* regions are preserved.

### UEFI Protocols

Common UEFI protocols used:
- `SimpleFileSystem` - File I/O
- `LoadedImage` - Bootloader info
- `GraphicsOutput` - Video (GOP)
- `SimpleTextInput` - Keyboard

### No Standard Library

Ignite is `no_std`:
- No `std::` - use `core::`  
- No heap by default - use `alloc::` after allocator setup
- No panic handler - must provide custom
- No main - use `#[no_mangle] pub extern "C" fn efi_main()`

## Performance Tips

1. **Minimize allocations** - Reuse buffers when possible
2. **Batch I/O** - Read large chunks, not byte-by-byte
3. **Lazy loading** - Only load what's needed
4. **Profile with `perf`** - Identify bottlenecks

## Security Considerations

See [SECURITY.md](../SECURITY.md) for full policy.

**Key points:**
- Validate all input (files, config, user input)
- Use safe APIs when possible
- Document unsafe code thoroughly
- Never trust user-provided data
- Implement defense in depth

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Build release binaries
5. Test on real hardware
6. Tag release: `git tag v0.x.0`
7. Push: `git push --tags`
8. Create GitHub release

## Troubleshooting

### Common Build Errors

**Error**: `unresolved import`
- **Fix**: Add dependency or fix module path

**Error**: `use of undeclared crate or module`
- **Fix**: Add `extern crate` or check `use` statement

**Error**: `can't find crate for 'std'`
- **Fix**: Ensure `#![no_std]` and using correct target

### QEMU Issues

**QEMU won't boot:**
- Check OVMF path is correct
- Verify ignite.efi is at `EFI/BOOT/BOOTX64.EFI`
- Check ESP filesystem is FAT32

**Serial output not showing:**
- Add `-serial stdio` to QEMU command
- Check bootloader is writing to serial

## Resources

- [Rust UEFI book](https://rust-osdev.github.io/uefi-rs/)
- [UEFI Specification](https://uefi.org/specifications)
- [OSDev Wiki](https://wiki.osdev.org/)
- [Rust Embedded Book](https://rust-embedded.github.io/book/)

## Getting Help

- Check existing documentation
- Search GitHub issues
- Ask in project discussions
- Join community chat (if available)

---

**Last Updated:** 2025-12-18  
**Version:** 0.4.0
