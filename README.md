# Ignite - Bootloader UEFI do Redstone OS

**Versão**: 0.4.0  
**Linguagem**: Rust (no_std)  
**Arquitetura**: x86_64 (UEFI)  
**Status**: Em desenvolvimento ativo (Funcional para Boot UEFI/FAT32)  

Ignite é o bootloader oficial do Redstone OS, escrito em Rust puro. O objetivo é fornecer uma inicialização rápida, segura e moderna, abstraindo a complexidade do hardware e entregando ao kernel um ambiente limpo e bem definido.

> ⚠️ **Nota:** A documentação anterior listava recursos planejados (como suporte a Multiboot, Linux Boot Protocol e sistema de configuração avançado) que ainda não estão ativos na branch principal. Este documento reflete o **estado atual real** do código.

## ✨ Funcionalidades Atuais

*   ✅ **UEFI Nativo:** Escrito especificamente para sistemas UEFI modernos (64-bit).
*   ✅ **Carregamento ELF64:** Faz o parse e carrega kernels no formato ELF64 na memória.
*   ✅ **Suporte a InitFS:** Carrega o sistema de arquivos inicial (`initfs`) para o kernel.
*   ✅ **Carregamento via FAT32:** Utiliza os protocolos da UEFI (`SimpleFileSystem`) para carregar arquivos da ESP.
*   ✅ **Configuração de Vídeo (GOP):** Permite ao usuário selecionar a resolução de vídeo antes do boot.
*   ✅ **Handover de Hardware:** Passa informações críticas para o kernel:
    *   Mapa de Memória (UEFI Memory Map).
    *   Tabelas ACPI (RSDP) e Device Tree.
    *   Buffers de vídeo (Framebuffer).
*   ✅ **Zero Dependencies (Runtime):** Não depende de libc, rodando 'bare metal' sobre a UEFI.

## 🚀 Fluxo de Boot (Atual)

1.  **POST & Firmware:** A placa-mãe carrega o arquivo `EFI/BOOT/BOOTX64.EFI` (o Ignite) da partição ESP.
2.  **Inicialização:** O Ignite inicializa serviços básicos de UEFI e alocadores de memória.
3.  **Seleção de Vídeo:** O Ignite lista os modos de vídeo disponíveis.
    *   O usuário seleciona o modo desejado (Setas + Enter).
4.  **Carregamento do Sistema:**
    *   Procura e carrega `boot/kernel` (Kernel do Redstone).
    *   Procura e carrega `boot/initfs` (Ramdisk inicial).
    *   *Nota: Atualmente usa a partição FAT32 de boot, contornando o driver RedstoneFS temporariamente.*
5.  **Salto para o Kernel:** O Ignite configura a stack, mapeia a memória e transfere a execução para o `_start` do Kernel.

## 🛠️ Como Compilar

Certifique-se de ter o Rust (nightly) instalado e o target UEFI adicionado.

```bash
# 1. Adicionar suporte a UEFI
rustup target add x86_64-unknown-uefi

# 2. Compilar (Debug)
cargo build --package ignite --target x86_64-unknown-uefi

# 3. Compilar (Release - Otimizado)
cargo build --package ignite --target x86_64-unknown-uefi --release
```

O binário será gerado em: `target/x86_64-unknown-uefi/debug/ignite.efi`.

## 📦 Estrutura de Arquivos (Target)

Para que o boot funcione corretamente (por exemplo, no QEMU ou hardware real), a estrutura de arquivos na partição bootável (ESP - FAT32) deve ser:

```text
/
├── EFI/
│   └── BOOT/
│       └── BOOTX64.EFI   <-- O binário do Ignite (renomeado)
└── boot/
    ├── kernel            <-- O kernel 'forge'
    └── initfs            <-- O arquivo 'initramfs.tar' (renomeado)
```

> O script de build `anvil.ps1` já prepara essa estrutura automaticamente em `dist/qemu/`.

## 🗺️ Roadmap e Desenvolvimento

### Em andamento
*   [ ] **RedstoneFS:** Implementação completa do driver de leitura para o sistema de arquivos nativo (ZFS-like).
*   [ ] **Configuração (ignite.conf):** Reabilitar o parser de configuração para não depender de caminhos *hardcoded*.
*   [ ] **Entrada Não-Bloqueante:** Melhorar a detecção de input para suportar Serial e Teclado simultaneamente sem "congelar" a interface gráfica.

### Planejado
*   [ ] Suporte a múltiplos protocolos (Multiboot2, Linux).
*   [ ] Verificação de integridade (Hash/Assinatura).
*   [ ] Shell de recuperação integrado.

## 🧩 Arquitetura do Código

*   `src/main.rs`: Ponto de entrada e lógica principal de boot.
*   `src/os/uefi/`: Implementação da abstração de SO para UEFI (Input, Vídeo, Filesystem).
*   `src/os/mod.rs`: Trait `Os` que define a interface comum para o bootloader.
*   `src/redstonefs.rs`: Driver (atualmente parcial/stub) para o sistema de arquivos RedstoneFS.

---

**Redstone OS Project**
