# Troubleshooting - Ignite Bootloader

Guia de solução de problemas comuns.

## 📋 Índice

- [Problemas de Compilação](#problemas-de-compilação)
- [Problemas de Boot](#problemas-de-boot)
- [Problemas de Configuração](#problemas-de-configuração)
- [Problemas de Vídeo](#problemas-de-vídeo)
- [Depuração Avançada](#depuração-avançada)

---

## Problemas de Compilação

### Erro: "target 'x86_64-unknown-uefi' not found"

**Causa**: Target UEFI não instalado.

**Solução**:
```bash
rustup target add x86_64-unknown-uefi --toolchain nightly
```

---

### Erro: "nightly toolchain required"

**Causa**: Usando toolchain stable em vez de nightly.

**Solução**:
```bash
rustup default nightly
# OU
rustup override set nightly
```

---

### Erro: "feature `abi_efiapi` is unstable"

**Causa**: Usando Rust stable ou toolchain desatualizada.

**Solução**:
```bash
rustup update nightly
rustup default nightly
cargo clean
cargo build --target x86_64-unknown-uefi
```

---

## Problemas de Boot

### Bootloader não executa (tela preta)

**Possíveis Causas**:

1. **Secure Boot habilitado**
   - Solução: Desabilitar Secure Boot no UEFI
   - Ou: Assinar o binário com chave válida

2. **Binário não está na localização correta**
   - ESP deve ter: `\EFI\BOOT\BOOTX64.EFI`
   - Verificar com:
     ```bash
     sudo mount /dev/sda1 /mnt
     ls -la /mnt/EFI/BOOT/
     ```

3. **ESP não é FAT32**
   - Verificar:
     ```bash
     sudo file -s /dev/sda1
     ```
   - Deve mostrar: `FAT (32 bit)`

---

### Panic: "Out of Memory (OOM)"

**Causa**: Heap inicial muito pequeno ou alocação excessiva.

**Solução**:
1. Aumentar heap em `src/main.rs`:
   ```rust
   let heap_size = 4 * 1024 * 1024; // 4 MiB em vez de 2 MiB
   ```
2. Verificar se há leaks de memória

---

### Panic: "Kernel não encontrado no disco"

**Causa**: Path em `ignite.conf` está incorreto.

**Solução**:
```ini
# Verificar caminho
path = "boot():/EFI/ignite/forge"  # ✅ Correto
# Não:
path = "/forge"  # ❌ Não encontra
```

Testar manualmente:
```bash
ls /mnt/esp/EFI/ignite/forge
```

---

### Erro: "Invalid use of a reserved firmware watchdog code"

**Causa**: Código de watchdog inválido passado ao firmware UEFI.

**Solução**: Já corrigido na versão atual. Se persistir, atualizar para versão mais recente.

---

### Page Fault após "Exit Boot Services"

**Possíveis Causas**:

1. **Paging não configurado corretamente**
   - Verificar CR3 carregado com PML4 válido

2. **Jump para endereço inválido**
   - Verificar entry_point do kernel:
     ```rust
     println!("Entry point: {:#x}", launch_info.entry_point);
     # Deve ser um endereço válido (ex: 0xFFFFFFFF80100000)
     ```

3. **Stack inválida**
   - Se protocolo requer stack, verificar se foi alocada

**Debug**:
```bash
# QEMU com monitor
qemu-system-x86_64 ... -monitor stdio
# Quando ocorrer fault:
(qemu) info registers
(qemu) x/16gx $rsp  # Ver stack
```

---

## Problemas de Configuração

### Erro: "Nenhuma entrada encontrada"

**Causa**: `ignite.conf` vaz io ou malformado.

**Solução**:
1. Verificar sintaxe:
   ```ini
   [[entry]]  # ✅ Correto
   name = "Test"
   protocol = "redstone"
   path = "boot():/kernel"
   ```

2. Garantir que há pelo menos um `[[entry]]`

---

### Timeout não funciona

**Causa**: Valor inválido ou conflito.

**Solução**:
```ini
timeout = 5      # ✅ Correto (inteiro)
# Não:
timeout = "5"    # ❌ String
```

---

### Protocolo não detectado

**Causa**: Magic bytes incorretos ou protocolo não implementado.

**Solução**:
1. Especificar protocolo explicitamente:
   ```ini
   protocol = "redstone"  # Não deixar auto-detect
   ```

2. Verificar magic bytes do kernel:
   ```bash
   hexdump -C kernel | head -n 1
   # ELF: 7f 45 4c 46 ...
   # Linux: 53 72 64 48 (em offset especifico)
   ```

---

## Problemas de Vídeo

### Menu não aparece (GOP initialization failed)

**Causa**: GOP não disponível ou firmware não suporta.

**Solução**:
1. Testar em QEMU primeiro:
   ```bash
   qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd ...
   ```

2. Se falhar em hardware real:
   - Atualizar firmware UEFI
   - Verificar se placa de vídeo suporta UEFI GOP

---

### Resolução incorreta

**Causa**: Resolução solicitada não suportada.

**Solução**:
1. Remover linha `resolution` do config (usa máxima disponível)
2. Ou testar com resolução mais baixa:
   ```ini
   resolution = 1024x768
   ```

---

### Framebuffer corrompido ou cores erradas

**Causa**: Formato de pixel incorreto (RGB vs BGR).

**Solução**: Bug no código. Reportar issue com:
- Firmware UEFI (versão)
- Placa de vídeo
- Screenshot do problema

---

## Problemas de Segurança

### Secure Boot bloqueia bootloader

**Solução Temporária**:
1. Desabilitar Secure Boot no UEFI

**Solução Permanente**:
1. Assinar o binário com chave própria
2. Adicionar chave ao UEFI DB

Instruções: Ver `docs/SEGURANCA.md`

---

### TPM não detectado

**Causa**: Hardware sem TPM ou TPM desabilitado.

**Solução**:
- Se TPM não é obrigatório: Configurar política para avisar apenas
- Se TPM é obrigatório: Habilitar TPM no UEFI

---

## Depuração Avançada

### Capturar Logs Serial

```bash
# QEMU
qemu-system-x86_64 ... -serial file:serial.log

# Hardware real (cabo serial USB-TTL)
screen /dev/ttyUSB0 115200
```

---

### GDB Remote Debugging

```bash
# Terminal 1: QEMU com gdbserver
qemu-system-x86_64 -s -S ...

# Terminal 2: GDB
gdb target/x86_64-unknown-uefi/debug/ignite.efi
(gdb) target remote :1234
(gdb) break efi_main
(gdb) continue
```

---

### Analisar Memory Dump

```bash
# QEMU monitor
(qemu) pmemsave 0 0x10000000 memdump.bin

# Análise
hexdump -C memdump.bin | less
```

---

### Verificar Integridade do Binário

```bash
# Deve ser PE32+
file target/x86_64-unknown-uefi/release/ignite.efi
# Output: PE32+ executable (EFI application) x86-64

# Ver seções
objdump -h target/x86_64-unknown-uefi/release/ignite.efi

# Verificar entry point
objdump -f target/x86_64-unknown-uefi/release/ignite.efi
```

---

## FAQ

**P: Por que o bootloader é tão lento no primeiro boot?**
R: UEFI pode fazer enumeração de hardware. Boots subsequentes são mais rápidos.

**P: Posso usar com Secure Boot?**
R: Sim, mas o binário precisa ser assinado com chave válida.

**P: Funciona com BIOS Legacy?**
R: Não. Ignite é UEFI-only.

**P: Como reportar bugs?**
R: GitHub Issues com logs serial e configuração completa.

---

**Última Atualização**: 2025-12-21
