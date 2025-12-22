# Guia de Configuração - ignite.conf

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Formato do Arquivo](#formato-do-arquivo)
- [Configurações Globais](#configurações-globais)
- [Entradas de Boot](#entradas-de-boot)
- [Protocolos Suportados](#protocolos-suportados)
- [Exemplos Práticos](#exemplos-práticos)
- [Resolução de Caminhos](#resolução-de-caminhos)
- [Troubleshooting](#troubleshooting)

---

## Visão Geral

O arquivo `ignite.conf` é o arquivo de configuração principal do bootloader Ignite. Ele define:

- Tempo de timeout do menu
- Entrada padrão de boot
- Resolução de vídeo
- Lista de sistemas operacionais disponíveis
- Parâmetros específicos de cada entrada

**Localização Padrão**: `boot():/EFI/ignite/ignite.conf`

---

## Formato do Arquivo

O formato é inspirado em TOML, mas usa um parser customizado simplificado.

### Sintaxe Básica

```ini
# Comentários começam com #
# Linhas vazias são ignoradas

# Configurações globais (uma por linha)
chave = valor

# Arrays de entradas
[[entry]]
campo1 = "valor1"
campo2 = "valor2"

[[entry]]
campo1 = "outro_valor"
```

### Tipos de Valores

```ini
# String (com aspas)
name = "Redstone OS"

# Número inteiro
timeout = 5

# Booleano
quiet = true
quiet = false

# Resolução (formato especial)
resolution = 1920x1080

# Array de módulos (dentro de [[entry]])
[[entry.module]]
path = "boot():/initrd.img"
```

---

## Configurações Globais

### timeout

**Tipo**: Integer  
**Padrão**: 5  
**Descrição**: Tempo em segundos antes de iniciar a entrada padrão automaticamente.

```ini
timeout = 10        # Aguardar 10 segundos
timeout = 0         # Boot imediato (sem menu)
timeout = -1        # Aguardar indefinidamente
```

> **Nota**: Se `timeout = 0`, o menu não será exibido e a entrada padrão será iniciada imediatamente.

---

### default

**Tipo**: Integer  
**Padrão**: 0  
**Descrição**: Índice da entrada padrão (baseado em 0).

```ini
default = 0    # Primeira entrada
default = 1    # Segunda entrada
default = 2    # Terceira entrada
```

> **Atenção**: Se o índice for inválido, o bootloader usará 0 automaticamente.

---

### quiet

**Tipo**: Boolean  
**Padrão**: false  
**Descrição**: Suprime logs não críticos.

```ini
quiet = false    # Mostrar todos os logs (recomendado para debug)
quiet = true     # Apenas mensagens críticas
```

---

### serial

**Tipo**: Boolean  
**Padrão**: true  
**Descrição**: Habilita saída serial (COM1).

```ini
serial = true     # Logs via COM1 (útil para debugging)
serial = false    # Desabilitar serial
```

> **Dica**: Deixe `serial = true` durante desenvolvimento para capturar logs via `qemu -serial stdio`.

---

### resolution

**Tipo**: String (formato `WIDTHxHEIGHT`)  
**Padrão**: Resolução máxima suportada pelo GOP  
**Descrição**: Resolução de vídeo desejada.

```ini
resolution = 1920x1080    # Full HD
resolution = 2560x1440    # 2K
resolution = 3840x2160    # 4K
resolution = 1024x768     # XGA (compatibilidade)
```

> **Nota**: Se a resolução solicitada não for suportada, a máxima disponível será usada.

---

### wallpaper

**Tipo**: String (caminho)  
**Padrão**: null  
**Descrição**: Caminho para imagem de fundo do menu (BMP 24-bit).

```ini
wallpaper = "boot():/EFI/ignite/background.bmp"
```

> **Futuro**: Suporte a PNG e JPEG planejado.

---

## Entradas de Boot

Cada entrada representa um sistema operacional ou aplicativo inicializável.

### Estrutura Básica

```ini
[[entry]]
name = "Nome Exibido no Menu"
protocol = "tipo_de_protocolo"
path = "caminho/para/kernel"
cmdline = "argumentos do kernel"          # Opcional
```

---

### Campos Obrigatórios

#### name

**Tipo**: String  
**Descrição**: Nome exibido no menu de boot.

```ini
name = "Redstone OS (Stable)"
name = "Redstone OS (Debug)"
name = "Arch Linux"
name = "Windows Boot Manager"
```

#### protocol

**Tipo**: String  
**Valores aceitos**: `redstone`, `linux`, `multiboot2`, `chainload`, `limine`, `native`

**Descrição**: Protocolo de boot a ser usado.

```ini
protocol = "redstone"    # Protocolo nativo do Redstone OS
protocol = "linux"       # Linux Boot Protocol (bzImage)
protocol = "multiboot2"  # Multiboot2 Specification
protocol = "chainload"   # UEFI LoadImage/StartImage
```

> **Aliases**: `limine` e `native` são sinônimos de `redstone`.

#### path

**Tipo**: String (caminho)  
**Descrição**: Caminho para o kernel ou aplicativo.

```ini
path = "boot():/EFI/ignite/forge"              # Kernel Redstone
path = "boot():/vmlinuz-linux"                  # Kernel Linux
path = "boot():/EFI/BOOT/shellx64.efi"         # UEFI Shell
path = "root():/boot/kernel.elf"                # Caminho alternativo
```

---

### Campos Opcionais

#### cmdline

**Tipo**: String  
**Descrição**: Argumentos de linha de comando passados ao kernel.

```ini
cmdline = "quiet splash"                         # Linux quiet mode
cmdline = "debug loglevel=7"                     # Debug verboso
cmdline = "root=/dev/sda1 init=/sbin/init"      # Root filesystem
cmdline = "--verbose --test-mode"                # Flags customizadas
```

---

### Módulos (Initrd, Drivers)

Cada entrada pode carregar módulos adicionais (initramfs, drivers, etc).

```ini
[[entry]]
name = "Sistema com InitRD"
protocol = "linux"
path = "boot():/vmlinuz"

[[entry.module]]
path = "boot():/initrd. img"
cmdline = "initrd"                               # Tag opcional

[[entry.module]]
path = "boot():/microcode.img"
cmdline = "microcode"
```

---

### Device Tree Blob (DTB)

Para arquiteturas ARM/RISC-V (futuro).

```ini
[[entry]]
name = "Redstone OS (ARM64)"
protocol = "redstone"
path = "boot():/forge-arm64"
dtb_path = "boot():/dtb/rpi4.dtb"                # Árvore de dispositivos
```

---

## Protocolos Suportados

### Redstone (Nativo)

**Formato**: ELF64  
**Magic Bytes**: `0x7F ELF`

**Descrição**: Protocolo nativo otimizado para kernels Redstone e compatíveis com Limine.

**Exemplo**:
```ini
[[entry]]
name = "Redstone OS"
protocol = "redstone"
path = "boot():/EFI/ignite/forge"
cmdline = "--verbose"
```

**Handoff**:
- RDI: Ponteiro para estrutura `BootInfo`
- Kernel carregado no higher-half (`0xFFFFFFFF80000000`)
- Framebuffer, memory map e ACPI tables fornecidos

---

### Linux Boot Protocol

**Formato**: bzImage (compressed kernel)  
**Magic Bytes**: `0x53726448` (no setup header)

**Descrição**: Carrega kernels Linux padrão.

**Exemplo**:
```ini
[[entry]]
name = "Arch Linux"
protocol = "linux"
path = "boot():/vmlinuz-linux"
cmdline = "root=/dev/sda2 rw quiet"

[[entry.module]]
path = "boot():/initramfs-linux.img"
```

**Handoff**:
- RSI: Ponteiro para `boot_params` structure
- Initrd carregado em memória alta
- Command line configurada no boot_params

---

### Multiboot2

**Formato**: ELF ou binário com header Multiboot2  
**Magic Bytes**: `0xE85250D6`

**Descrição**: Compatibilidade com kernels Multiboot2 (ex: GRUB modules).

**Exemplo**:
```ini
[[entry]]
name = "Multiboot2 Kernel"
protocol = "multiboot2"
path = "boot():/kernel.elf"
cmdline = "debug"

[[entry.module]]
path = "boot():/module1.ko"
```

**Handoff**:
- RBX: Ponteiro para MBI (Multiboot Information)
- RAX: Magic number `0x36D76289`
- Tags configuradas conforme spec

---

### UEFI Chainload

**Formato**: Binário PE32+ (executável UEFI)  
**Magic Bytes**: `MZ` (DOS header)

**Descrição**: Executa outro aplicativo UEFI (ex: UEFI Shell, outro bootloader).

**Exemplo**:
```ini
[[entry]]
name = "UEFI Shell"
protocol = "chainload"
path = "boot():/EFI/BOOT/shellx64.efi"

[[entry]]
name = "Windows Boot Manager"
protocol = "chainload"
path = "boot():/EFI/Microsoft/Boot/bootmgfw.efi"
```

**Comportamento**:
- Usa `LoadImage()` e `StartImage()` do UEFI
- Mantém Boot Services ativos
- Se o app retornar, o Ignite reinicia ou volta ao menu

---

## Exemplos Práticos

### Configuração Mínima

```ini
# ignite.conf mínimo
timeout = 5
default = 0

[[entry]]
name = "Redstone OS"
protocol = "redstone"
path = "boot():/EFI/ignite/forge"
```

---

### Configuração Completa (Multi-Boot)

```ini
# ============================================================================
# Ignite Bootloader Configuration
# Redstone OS Multi-Boot Setup
# ============================================================================

# --- Configurações Globais ---
timeout = 10
default = 0
quiet = false
serial = true
resolution = 1920x1080
wallpaper = "boot():/EFI/ignite/redstone-bg.bmp"

# --- Entrada 1: Redstone OS (Produção) ---
[[entry]]
name = "Redstone OS (Stable)"
protocol = "redstone"
path = "boot():/EFI/ignite/forge"
cmdline = "quiet"

# --- Entrada 2: Redstone OS (Debug) ---
[[entry]]
name = "Redstone OS (Debug Mode)"
protocol = "redstone"
path = "boot():/EFI/ignite/forge-debug"
cmdline = "--verbose --log-level=trace"

# --- Entrada 3: Arch Linux ---
[[entry]]
name = "Arch Linux"
protocol = "linux"
path = "boot():/vmlinuz-linux"
cmdline = "root=/dev/nvme0n1p2 rw quiet splash"

[[entry.module]]
path = "boot():/initramfs-linux.img"
cmdline = "initrd"

[[entry.module]]
path = "boot():/intel-ucode.img"
cmdline = "microcode"

# --- Entrada 4: Arch Linux (Fallback) ---
[[entry]]
name = "Arch Linux (Fallback)"
protocol = "linux"
path = "boot():/vmlinuz-linux"
cmdline = "root=/dev/nvme0n1p2 rw"

[[entry.module]]
path = "boot():/initramfs-linux-fallback.img"

# --- Entrada 5: Memtest86+ ---
[[entry]]
name = "Memory Test (Memtest86+)"
protocol = "multiboot2"
path = "boot():/memtest86+/memtest.bin"

# --- Entrada 6: UEFI Shell (Recovery) ---
[[entry]]
name = "UEFI Shell (Recovery)"
protocol = "chainload"
path = "boot():/EFI/BOOT/shellx64.efi"

# --- Entrada 7: UEFI Firmware Settings ---
[[entry]]
name = "Reboot to UEFI Firmware"
protocol = "chainload"
path = "boot():/EFI/tools/firmware-setup.efi"
```

---

### Configuração com Múltiplos Discos

```ini
# Boot de múltiplos dispositivos
timeout = 5
default = 0

# Kernel no ESP
[[entry]]
name = "Redstone OS (ESP)"
protocol = "redstone"
path = "boot():/EFI/ignite/forge"

# Kernel na partição raiz
[[entry]]
name = "Redstone OS (Root)"
protocol = "redstone"
path = "root():/boot/forge"
cmdline = "root=/dev/sda2"
```

---

## Resolução de Caminhos

O Ignite suporta esquemas de URL customizados para resolver caminhos.

### boot():/ (ESP)

Resolve para a **ESP (EFI System Partition)** de onde o bootloader foi carregado.

```ini
path = "boot():/EFI/ignite/forge"
# Equivalente a: \EFI\ignite\forge na ESP
```

**Quando usar**:
- Arquivos na ESP (kernels, initrd, config)
- Aplicativos UEFI
- Qualquer arquivo acessível via SimpleFileSystem

---

### root():/ (Root FS)

Resolve para a **partição raiz montada** (configurável).

```ini
path = "root():/boot/vmlinuz"
# Equivalente a: /boot/vmlinuz no sistema de arquivos raiz
```

**Quando usar**:
- Kernels instalados em `/boot`
- Configurações do sistema operacional

> **Nota**: O Ignite precisa conseguir montar a partição raiz. Atualmente, apenas FAT32 é suportado nativamente.

---

### Caminhos Absolutos

Caminhos sem prefixo são relativos ao diretório atual (geralmente ESP root).

```ini
path = "/EFI/ignite/forge"
# Relativo à raiz do filesystem atual
```

---

### Caminhos Relativos

```ini
path = "kernel/forge"
# Relativo ao diretório onde está ignite.conf
```

---

## Troubleshooting

### Erro: "Arquivo não encontrado"

**Causa**: Caminho incorreto no `path`.

**Solução**:
1. Verificar se o arquivo existe:
   ```bash
   ls /mnt/esp/EFI/ignite/
   ```
2. Conferir o prefixo (`boot():/`, `root():/`)
3. Verificar maiúsculas/minúsculas (FAT32 é case-insensitive, mas o parser pode não ser)

---

### Erro: "Nenhuma entrada encontrada"

**Causa**: O arquivo `ignite.conf` está vazio ou malformado.

**Solução**:
1. Verificar sintaxe do arquivo
2. Garantir que há pelo menos um `[[entry]]`
3. Ver logs serial para detalhes do parser

---

### Erro: "Protocolo desconhecido"

**Causa**: Valor inválido no campo `protocol`.

**Solução**:
Usar um dos valores aceitos:
- `redstone` / `limine` / `native`
- `linux`
- `multiboot2`
- `chainload`

---

### Timeout não funciona

**Causa**: Valor inválido ou conflito com `quiet`.

**Solução**:
```ini
timeout = 5         # Deve ser um número inteiro
quiet = false       # Se quiet=true, menu pode ser suprimido
```

---

### Resolução não aplicada

**Causa**: Resolução não suportada pela GOP.

**Solução**:
1. Testar com resolução padrão primeiro (remover linha `resolution`)
2. Verificar resoluções suportadas via UEFI Shell:
   ```
   mode
   ```
3. Escolher uma resolução listada

---

## Validação de Configuração

### Ferramenta de Validação (futuro)

```bash
# Validar sintaxe sem boot
ignite-config-check ignite.conf
```

### Validação Manual

Checklist:
- [ ] Arquivo possui extensão `.conf`
- [ ] Pelo menos uma entrada `[[entry]]` definida
- [ ] Cada entrada tem `name`, `protocol` e `path`
- [ ] Valores de `protocol` são válidos
- [ ] Caminhos existem na ESP/Root FS
- [ ] Índice `default` é válido (< número de entradas)
- [ ] Timeout está entre -1 e 300

---

## Migração de GRUB

### Converter grub.cfg para ignite.conf

**GRUB**:
```grub
menuentry "Linux" {
    linux /vmlinuz root=/dev/sda1 ro
    initrd /initrd.img
}
```

**Ignite**:
```ini
[[entry]]
name = "Linux"
protocol = "linux"
path = "boot():/vmlinuz"
cmdline = "root=/dev/sda1 ro"

[[entry.module]]
path = "boot():/initrd.img"
```

---

**Última Atualização**: 2025-12-21  
**Versão do Documento**: 1.0  
**Mantenedor**: Redstone OS Team
