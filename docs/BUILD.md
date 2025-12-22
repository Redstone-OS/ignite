# Guia de Build e Deployment - Ignite Bootloader

## 📋 Índice

- [Pré-requisitos](#pré-requisitos)
- [Processo de Build](#processo-de-build)
- [Deployment](#deployment)
- [Criação de Imagens Bootáveis](#criação-de-imagens-bootáveis)
- [Testes em QEMU](#testes-em-qemu)
- [Instalação em Hardware Real](#instalação-em-hardware-real)
- [Automação](#automação)

---

## Pré-requisitos

### Software Necessário

```bash
# Rust nightly
rustup toolchain install nightly
rustup default nightly

# Target UEFI
rustup target add x86_64-unknown-uefi

# Ferramentas buildessenciais
sudo apt install build-essential git

# QEMU (para testes)
sudo apt install qemu-system-x86 ovmf

# Ferramentas de manipulação de disco
sudo apt install mtools dosfstools parted
```

---

## Processo de Build

### Build Debug

```bash
cd ignite

# Compilação debug (com símbolos)
cargo build --target x86_64-unknown-uefi

# Binário gerado em:
# target/x86_64-unknown-uefi/debug/ignite.efi
```

**Características**:
- Tamanho: ~800 KB
- Otimização: Nenhuma
- Símbolos de debug: Sim
- Uso: Desenvolvimento e debugging

---

### Build Release

```bash
# Compilação release (otimizada)
cargo build --release --target x86_64-unknown-uefi

# Binário gerado em:
# target/x86_64-unknown-uefi/release/ignite.efi
```

**Características**:
- Tamanho: ~200-300 KB
- Otimização: Máxima (size)
- Símbolos de debug: Não
- Stripped: Sim
- LTO: Habilitado
- Uso: Produção

---

### Verificação do Build

```bash
# Ver tamanho do binário
ls -lh target/x86_64-unknown-uefi/release/ignite.efi

# Verificar que é um binário PE válido
file target/x86_64-unknown-uefi/release/ignite.efi
# Output esperado: PE32+ executable (EFI application) x86-64

# Ver seções do binário
cargo objdump --target x86_64-unknown-uefi --release -- -h

# Analisar tamanho por função
cargo bloat --target x86_64-unknown-uefi --release
```

---

## Deployment

### Estrutura de Diretórios na ESP

```
ESP (EFI System Partition - FAT32)
├── EFI/
│   ├── BOOT/
│   │   └── BOOTX64.EFI          ← Bootloader (fallback UEFI)
│   │
│   └── ignite/
│       ├── ignite.efi            ← Bootloader (cópia nomeada)
│       ├── ignite.conf           ← Configuração
│       ├── forge                 ← Kernel Redstone
│       ├── vmlinuz-linux         ← Kernel Linux (opcional)
│       ├── initramfs-linux.img   ← Initrd Linux (opcional)
│       └── background.bmp        ← Wallpaper (opcional)
```

---

### Copiar Binário para ESP

#### Método 1: Montagem Manual

```bash
# Identificar partição ESP
lsblk
# Exemplo: /dev/sda1 é a ESP

# Montar ESP
sudo mkdir -p /mnt/esp
sudo mount /dev/sda1 /mnt/esp

# Criar diretórios
sudo mkdir -p /mnt/esp/EFI/BOOT
sudo mkdir -p /mnt/esp/EFI/ignite

# Copiar bootloader
sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/esp/EFI/BOOT/BOOTX64.EFI
sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/esp/EFI/ignite/

# Sincronizar e desmontar
sudo sync
sudo umount /mnt/esp
```

#### Método 2: Script Automatizado

```bash
#!/bin/bash
# deploy.sh

set -e

ESP_PARTITION="/dev/sda1"
ESP_MOUNT="/mnt/esp"
BUILD_TYPE="release"

# Build
echo "Building Ignite..."
cargo build --$BUILD_TYPE --target x86_64-unknown-uefi

# Montar ESP
echo "Mounting ESP..."
sudo mkdir -p $ESP_MOUNT
sudo mount $ESP_PARTITION $ESP_MOUNT

# Criar estrutura
echo "Creating directory structure..."
sudo mkdir -p $ESP_MOUNT/EFI/BOOT
sudo mkdir -p $ESP_MOUNT/EFI/ignite

# Copiar binário
echo "Copying bootloader..."
sudo cp target/x86_64-unknown-uefi/$BUILD_TYPE/ignite.efi $ESP_MOUNT/EFI/BOOT/BOOTX64.EFI
sudo cp target/x86_64-unknown-uefi/$BUILD_TYPE/ignite.efi $ESP_MOUNT/EFI/ignite/

# Copiar configuração (se existir)
if [ -f "ignite.conf" ]; then
    echo "Copying configuration..."
    sudo cp ignite.conf $ESP_MOUNT/EFI/ignite/
fi

# Desmontar
echo "Unmounting..."
sudo sync
sudo umount $ESP_MOUNT

echo "✅ Deployment complete!"
```

```bash
chmod +x deploy.sh
sudo ./deploy.sh
```

---

### Configurar NVRAM (Boot Entry)

```bash
# Adicionar entrada de boot no NVRAM
sudo efibootmgr --create \
    --disk /dev/sda \
    --part 1 \
    --label "Ignite Bootloader" \
    --loader '\EFI\ignite\ignite.efi'

# Listar entradas
sudo efibootmgr -v

# Definir ordem de boot (exemplo: 0003 é o Ignite)
sudo efibootmgr --bootorder 0003,0000,0001

# Remover entrada (se necessário)
sudo efibootmgr --bootnum 0003 --delete-bootnum
```

---

## Criação de Imagens Bootáveis

### Imagem de Disco Raw

```bash
#!/bin/bash
# create_disk_image.sh

set -e

IMAGE="ignite-disk.img"
SIZE_MB=128

echo "Creating disk image ($SIZE_MB MB)..."
dd if=/dev/zero of=$IMAGE bs=1M count=$SIZE_MB

echo "Creating GPT partition table..."
parted $IMAGE mklabel gpt
parted $IMAGE mkpart primary fat32 1MiB 100%
parted $IMAGE set 1 esp on

echo "Setting up loop device..."
LOOP_DEV=$(sudo losetup -fP --show $IMAGE)
LOOP_PART="${LOOP_DEV}p1"

echo "Formatting ESP..."
sudo mkfs.vfat -F32 $LOOP_PART

echo "Mounting filesystem..."
sudo mkdir -p /mnt/esp
sudo mount $LOOP_PART /mnt/esp

echo "Copying bootloader..."
sudo mkdir -p /mnt/esp/EFI/BOOT
sudo mkdir -p /mnt/esp/EFI/ignite

sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/esp/EFI/BOOT/BOOTX64.EFI
sudo cp ignite.conf /mnt/esp/EFI/ignite/ || true
sudo cp /path/to/kernel /mnt/esp/EFI/ignite/forge || true

echo "Cleaning up..."
sudo umount /mnt/esp
sudo losetup -d $LOOP_DEV

echo "✅ Disk image created: $IMAGE"
echo "   Test with: qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -drive file=$IMAGE,format=raw"
```

---

### Imagem ISO (CD/DVD Bootável)

```bash
#!/bin/bash
# create_iso.sh

set -e

ISO_DIR="iso_root"
ISO_FILE="ignite-boot.iso"

echo "Creating ISO directory structure..."
mkdir -p $ISO_DIR/EFI/BOOT
mkdir -p $ISO_DIR/EFI/ignite

echo "Copying files..."
cp target/x86_64-unknown-uefi/release/ignite.efi $ISO_DIR/EFI/BOOT/BOOTX64.EFI
cp ignite.conf $ISO_DIR/EFI/ignite/ || true
cp /path/to/kernel $ISO_DIR/EFI/ignite/forge || true

echo "Creating ISO image..."
xorriso -as mkisofs \
    -R -J -joliet-long \
    -eltorito-alt-boot \
    -e /EFI/BOOT/BOOTX64.EFI \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    -V "IGNITE_BOOT" \
    -o $ISO_FILE \
    $ISO_DIR

echo "Cleaning up..."
rm -rf $ISO_DIR

echo "✅ ISO created: $ISO_FILE"
echo "   Test with: qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -cdrom $ISO_FILE"
```

---

### USB Bootável

```bash
#!/bin/bash
# create_usb.sh
# ATENÇÃO: Substitua /dev/sdX pelo dispositivo USB correto!

set -e

USB_DEVICE="/dev/sdX"  # ⚠️ CUIDADO: Verificar com lsblk
USB_PART="${USB_DEVICE}1"

echo "⚠️  WARNING: This will ERASE ALL DATA on $USB_DEVICE"
read -p "Continue? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo "Creating GPT partition table..."
sudo parted $USB_DEVICE mklabel gpt
sudo parted $USB_DEVICE mkpart primary fat32 1MiB 100%
sudo parted $USB_DEVICE set 1 esp on

echo "Formatting..."
sudo mkfs.vfat -F32 $USB_PART

echo "Mounting..."
sudo mkdir -p /mnt/usb
sudo mount $USB_PART /mnt/usb

echo "Copying bootloader..."
sudo mkdir -p /mnt/usb/EFI/BOOT
sudo mkdir -p /mnt/usb/EFI/ignite
sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/usb/EFI/BOOT/BOOTX64.EFI
sudo cp ignite.conf /mnt/usb/EFI/ignite/ || true

echo "Unmounting..."
sudo sync
sudo umount /mnt/usb

echo "✅ USB bootável criado em $USB_DEVICE"
```

---

## Testes em QEMU

### Configuração Básica do QEMU

```bash
#!/bin/bash
# run_qemu.sh

set -e

DISK_IMAGE="ignite-disk.img"
OVMF_CODE="/usr/share/ovmf/OVMF_CODE.fd"
OVMF_VARS="/usr/share/ovmf/OVMF_VARS.fd"

# Copiar VARS (permite modificações)
cp $OVMF_VARS ./OVMF_VARS_TEMP.fd

qemu-system-x86_64 \
    -enable-kvm \
    -cpu host \
    -m 4G \
    -smp 2 \
    -drive if=pflash,format=raw,readonly=on,file=$OVMF_CODE \
    -drive if=pflash,format=raw,file=./OVMF_VARS_TEMP.fd \
    -drive file=$DISK_IMAGE,format=raw \
    -serial stdio \
    -display gtk \
    -net none
```

---

### QEMU com Debug

```bash
qemu-system-x86_64 \
    -s \                    # GDB server na porta 1234
    -S \                    # Pausar no início
    -enable-kvm \
    -cpu host \
    -m 4G \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF.fd \
    -drive file=ignite-disk.img,format=raw \
    -serial stdio \
    -monitor telnet:127.0.0.1:55555,server,nowait \
    -display gtk
```

Em outro terminal:
```bash
# GDB
gdb target/x86_64-unknown-uefi/debug/ignite.efi
(gdb) target remote :1234
(gdb) break efi_main
(gdb) continue

# QEMU Monitor
telnet 127.0.0.1 55555
(qemu) info registers
(qemu) info mem
```

---

### QEMU com Logs Completos

```bash
qemu-system-x86_64 \
    -enable-kvm \
    -cpu host \
    -m 4G \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/ovmf/OVMF.fd \
    -drive file=ignite-disk.img,format=raw \
    -serial file:serial.log \    # Logs serial em arquivo
    -d int,cpu_reset \            # Debug de interrupções e resets
    -D qemu.log \                 # Arquivo de log do QEMU
    -display gtk
```

Analisar logs:
```bash
tail -f serial.log    # Logs do bootloader
less qemu.log         # Logs internos do QEMU
```

---

## Instalação em Hardware Real

### Pré-instalação

1. **Backup**: Faça backup completo do sistema
2. **Secure Boot**: Desabilite Secure Boot no UEFI (temporariamente)
3. **Identificar ESP**: Use `lsblk` ou `fdisk -l`

---

### Instalação

```bash
# 1. Identificar ESP
lsblk
# Exemplo: /dev/nvme0n1p1 (FAT32, ~512MB)

# 2. Montar
sudo mount /dev/nvme0n1p1 /mnt/esp

# 3. Backup do bootloader atual (segurança)
sudo cp -r /mnt/esp/EFI /mnt/esp/EFI.backup

# 4. Instalar Ignite
sudo mkdir -p /mnt/esp/EFI/ignite
sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/esp/EFI/ignite/
sudo cp ignite.conf /mnt/esp/EFI/ignite/

# 5. (Opcional) Substituir BOOTX64.EFI
# ⚠️ Cuidado: Isso tornará Ignite o bootloader padrão
sudo cp /mnt/esp/EFI/BOOT/BOOTX64.EFI /mnt/esp/EFI/BOOT/BOOTX64.EFI.backup
sudo cp target/x86_64-unknown-uefi/release/ignite.efi /mnt/esp/EFI/BOOT/BOOTX64.EFI

# 6. Adicionar entrada NVRAM
sudo efibootmgr --create \
    --disk /dev/nvme0n1 \
    --part 1 \
    --label "Ignite Bootloader" \
    --loader '\EFI\ignite\ignite.efi'

# 7. Verificar ordem de boot
sudo efibootmgr -v

# 8. Desmontar
sudo sync
sudo umount /mnt/esp
```

---

### Rollback em Caso de Problemas

Se o sistema não bootar:

1. **Bootar via USB Live**
2. **Montar ESP**:
   ```bash
   sudo mount /dev/nvme0n1p1 /mnt
   ```
3. **Restaurar backup**:
   ```bash
   sudo cp /mnt/EFI/BOOT/BOOTX64.EFI.backup /mnt/EFI/BOOT/BOOTX64.EFI
   ```
4. **Remover entrada NVRAM**:
   ```bash
   sudo efibootmgr --bootnum XXXX --delete-bootnum
   ```

---

## Automação

### Makefile

```makefile
# Makefile para Ignite Bootloader

.PHONY: all build build-release clean test deploy install-qemu run-qemu

# Configurações
TARGET := x86_64-unknown-uefi
RELEASE_DIR := target/$(TARGET)/release
DEBUG_DIR := target/$(TARGET)/debug

all: build-release

build:
	cargo build --target $(TARGET)

build-release:
	cargo build --release --target $(TARGET)

clean:
	cargo clean

test:
	cargo test --lib

# Deploy local (requer variável ESP_MOUNT)
deploy: build-release
	@if [ -z "$(ESP_MOUNT)" ]; then \
		echo "Erro: defina ESP_MOUNT (ex: make deploy ESP_MOUNT=/mnt/esp)"; \
		exit 1; \
	fi
	sudo mkdir -p $(ESP_MOUNT)/EFI/BOOT
	sudo mkdir -p $(ESP_MOUNT)/EFI/ignite
	sudo cp $(RELEASE_DIR)/ignite.efi $(ESP_MOUNT)/EFI/BOOT/BOOTX64.EFI
	sudo cp ignite.conf $(ESP_MOUNT)/EFI/ignite/ || true
	@echo "Deployed to $(ESP_MOUNT)"

# Variáveis para QEMU
IMAGE := disk.img
OVMF := /usr/share/ovmf/OVMF.fd

install-qemu: build-release
	./tools/create_disk_image.sh

run-qemu:
	qemu-system-x86_64 \
		-enable-kvm \
		-cpu host \
		-m 4G \
		-bios $(OVMF) \
		-drive file=$(IMAGE),format=raw \
		-serial stdio \
		-display gtk
```

Uso:
```bash
make build-release              # Compilar release
make deploy ESP_MOUNT=/mnt/esp  # Deploy para ESP
make install-qemu               # Criar imagem QEMU
make run-qemu                   # Executar em QEMU
```

---

### CI/CD (GitHub Actions)

```yaml
# .github/workflows/build.yml
name: Build Ignite Bootloader

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: nightly
        target: x86_64-unknown-uefi
        override: true
        components: rustfmt, clippy
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy --target x86_64-unknown-uefi -- -D warnings
    
    - name: Build (debug)
      run: cargo build --target x86_64-unknown-uefi
    
    - name: Build (release)
      run: cargo build --release --target x86_64-unknown-uefi
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: ignite-bootloader
        path: target/x86_64-unknown-uefi/release/ignite.efi
```

---

**Última Atualização**: 2025-12-21  
**Versão do Documento**: 1.0  
**Mantenedor**: Redstone OS Team
