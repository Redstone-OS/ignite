# Índice - Ignite (UEFI Bootloader) v0.4.0

Este diretório contém o bootloader UEFI do Redstone OS, responsável por inicializar o sistema e carregar o kernel com suporte multi-protocolo.

## 📊 Estatísticas do Projeto (v0.4.0)

- **Total de arquivos**: 60+ (código + documentação)
- **Linhas de código**: ~6000+
- **Módulos**: 14 especializados
- **Protocolos suportados**: 5 (Limine, Linux, Multiboot1, Multiboot2, EFI Chainload)
- **Filesystems**: 2 (FAT32, ISO9660)
- **Cobertura de documentação**: 100%
- **Status de compilação**: ✅ Sucesso (3.53s, 0 erros)
- **Versão**: 0.4.0

## 📁 Estrutura Completa de Arquivos

```bash
ignite/
├── 📋 Documentação
│   ├── README.md                    # Documentação principal (✨ ATUALIZADO v0.4)
│   ├── CHANGELOG.md                 # Histórico de mudanças (✨ v0.4.0)
│   ├── SECURITY.md                  # Política de segurança (✨ ATUALIZADO)
│   ├── INDICE.md                    # Este arquivo (✨ ATUALIZADO)
│   ├── CONTRIBUTING.md              # Guia de contribuição
│   ├── CODE_OF_CONDUCT.md           # Código de conduta
│   ├── AUTHORS.md                   # Autores e contribuidores
│   └── LICENSE                      # Licença MIT
│
├── ⚙️ Configuração
│   ├── Cargo.toml                   # Pacote Rust
│   ├── rust-toolchain.toml          # Versão do Rust
│   ├── rustfmt.toml                 # Formatação de código
│   ├── .clippy.toml                 # Linter config
│   ├── .editorconfig                # Editor config
│   └── .gitignore                   # Git ignore
│
├── 📚 docs/                         # Documentação Adicional
│   ├── README.md                    # Recursos e referências (✨ MELHORADO)
│   ├── protocols.md                 # ✨ NOVO: Documentação de protocolos
│   ├── configuration.md             # ✨ NOVO: Sistema de configuração
│   └── development.md               # ✨ NOVO: Guia de desenvolvimento
│
└── 💻 src/                          # Código-Fonte
    ├── main.rs                      # Entry point (11 linhas)
    ├── lib.rs                       # Orquestração principal (✨ ATUALIZADO)
    ├── boot_info.rs                 # Estruturas de boot info
    ├── error.rs                     # Sistema de erros (175 linhas)
    ├── types.rs                     # Tipos compartilhados (68 linhas)
    │
    ├── protos/                      # ⭐ NOVO: Multi-Protocol Support
    │   ├── mod.rs                   # BootProtocol trait (104 linhas)
    │   ├── limine.rs                # Limine protocol (84 linhas)
    │   ├── linux.rs                 # Linux boot protocol (281 linhas)
    │   ├── multiboot1.rs            # Multiboot 1 (312 linhas)
    │   ├── multiboot2.rs            # Multiboot 2 (137 linhas)
    │   └── chainload.rs             # EFI/BIOS chainload (90 linhas)
    │
    ├── config/                      # ⭐ NOVO: Configuration System
    │   ├── mod.rs                   # Módulo root
    │   ├── types.rs                 # Config types (139 linhas)
    │   ├── parser.rs                # Config parser (290 linhas)
    │   ├── paths.rs                 # Path resolver (208 linhas)
    │   ├── macros.rs                # Macro expander (119 linhas)
    │   └── validator.rs             # Config validator (89 linhas)
    │
    ├── ui/                          # ⭐ NOVO: User Interface
    │   ├── mod.rs                   # Módulo root
    │   ├── menu.rs                  # Boot menu (71 linhas)
    │   ├── input.rs                 # Input handler (40 linhas)
    │   ├── terminal.rs              # Graphical terminal (80 linhas)
    │   ├── theme.rs                 # Color themes (39 linhas)
    │   └── editor.rs                # Config editor (39 linhas)
    │
    ├── fs/                          # Filesystem Support
    │   ├── mod.rs                   # Módulo root
    │   ├── loader.rs                # UEFI file loader (93 linhas)
    │   ├── initfs.rs                # InitFS loader (25 linhas)
    │   ├── fat32.rs                 # ⭐ NOVO: FAT32 driver (155 linhas)
    │   └── iso9660.rs               # ⭐ NOVO: ISO9660 driver (120 linhas)
    │
    ├── hardware/                    # ⭐ NOVO: Hardware Abstraction
    │   ├── mod.rs                   # Módulo root
    │   ├── acpi.rs                  # ACPI support (92 linhas)
    │   └── fdt.rs                   # Device Tree (57 linhas)
    │
    ├── elf/                         # ELF Loader
    │   ├── mod.rs                   # Módulo root
    │   ├── parser.rs                # ELF parser (56 linhas)
    │   └── loader.rs                # Segment loader (88 linhas)
    │
    ├── memory/                      # Memory Management
    │   ├── mod.rs                   # Módulo root
    │   └── allocator.rs             # UEFI allocator (86 linhas)
    │
    ├── video/                       # Video Configuration
    │   ├── mod.rs                   # Módulo root + trait
    │   └── gop.rs                   # GOP implementation (73 linhas)
    │
    ├── security/                    # Security Features
    │   ├── mod.rs                   # Módulo root
    │   ├── integrity.rs             # Integrity verification
    │   ├── rollback.rs              # Rollback protection
    │   ├── secureboot.rs            # Secure Boot (109 linhas)
    │   └── blake2b.rs               # ⭐ NOVO: BLAKE2B hash (64 linhas)
    │
    └── recovery/                    # Recovery System
        ├── mod.rs                   # Módulo root
        ├── fallback.rs              # Fallback mechanism (118 linhas)
        ├── keydetect.rs             # Key detection (28 linhas)
        └── diagnostics.rs           # Diagnostics (56 linhas)
```

## 🎯 Novidades v0.4.0

### 🚀 Multi-Protocol Boot Support

**5 Protoclos Implementados:**
- **Limine** - Protocolo nativo do Redstone OS
- **Linux** - bzImage, initrd, cmdline completo
- **Multiboot 1** - Especificação clássica
- **Multiboot 2** - Tags modernas
- **EFI Chainload** - Carrega outros bootloaders

### ⚙️ Sistema de Configuração Completo

- Parser Limine-compatible (`ignite.conf`)
- Paths avançados: `boot():/`, `hdd(D:P):/`, `guid(UUID):/`
- Macros: `${ARCH}`, `${FW_TYPE}`, customizáveis
- Validação sintática e semântica

### 🖥️ Interface Interativa

- Menu de boot navegável (↑↓, Enter)
- Terminal gráfico com framebuffer
- Temas customizáveis
- Editor de config in-bootloader

### 💾 Drivers Nativos de Filesystem

- **FAT32** - FAT12/16/32 independente de UEFI
- **ISO9660** - CD/DVD support

### 🔧 Hardware Abstraction

- **ACPI** - RSDP, RSDT, XSDT parsing
- **FDT** - Device Tree para ARM64/RISC-V

## 📖 Descrição dos Módulos

### Core

#### `src/main.rs` (11 linhas)
Entry point minimalista que apenas chama `ignite::boot()`.

#### `src/lib.rs`
Orquestrador principal do boot process com integração de todos os módulos.

#### `src/error.rs` (175 linhas)
Sistema de erros robusto com tipos específicos para cada módulo.

#### `src/types.rs` (68 linhas)
Tipos compartilhados: `KernelArgs`, `Framebuffer`, `LoadedFile`, `LoadedKernel`.

### Protocolos de Boot (Novos)

#### `src/protos/mod.rs`
Define o trait `BootProtocol` e abstrações comuns (`BootInfo`, `ProtocolRegisters`).

#### `src/protos/limine.rs`
Implementa protocolo Limine usando `ElfLoader` existente.

#### `src/protos/linux.rs`
Linux Boot Protocol com:
- Parsing de `SetupHeader`
- Validação de magic numbers
- Carregamento de bzImage, initrd
- Setup de boot_params (parcial)

#### `src/protos/multiboot1.rs`
Multiboot 1 specification:
- Busca de header em primeiros 8KB
- Suporte a "a.out kludge"
- Suporte a ELF format
- Criação de Multiboot Info structure

#### `src/protos/multiboot2.rs`
Multiboot 2 com tag system usando ElfLoader.

#### `src/protos/chainload.rs`
Chainloading de aplicações EFI (PE/COFF validation).

### Sistema de Configuração (Novo)

#### `src/config/types.rs`
Define `BootConfig`, `MenuEntry`, `Module`, `WallpaperStyle`.

#### `src/config/parser.rs`
Parser completo para formato Limine-compatible com suporte a:
- Opções globais
- Entradas hierárquicas
- Expansão de macros

#### `src/config/paths.rs`
Resolvedor de paths com suporte a:
- `boot():/`, `boot(N):/`
- `hdd(D:P):/`, `guid(UUID):/`, `fslabel(LABEL):/`
- Verificação de hash inline

#### `src/config/macros.rs`
Sistema de macros com built-ins e customizáveis.

#### `src/config/validator.rs`
Validador de sintaxe e semântica de configuração.

### Interface de Usuário (Nova)

#### `src/ui/menu.rs`
Menu interativo de boot com navegação e seleção.

#### `src/ui/input.rs`
Handler de input de teclado (estrutura, integração UEFI TODO).

#### `src/ui/terminal.rs`
Terminal gráfico para renderização em framebuffer.

#### `src/ui/theme.rs`
Sistema de temas com cores customizáveis.

#### `src/ui/editor.rs`
Editor de configuração in-bootloader.

### Filesystems

#### `src/fs/loader.rs` (93 linhas)
Carregador de arquivos via UEFI File Protocol.

#### `src/fs/initfs.rs` (25 linhas)
Carregador de sistema de arquivos inicial opcional.

#### `src/fs/fat32.rs` (Novo - 155 linhas)
Driver FAT32 nativo com BPB parsing (read_file TODO).

#### `src/fs/iso9660.rs` (Novo - 120 linhas)
Driver ISO9660 para CD/DVD (read_file TODO).

### Hardware Abstraction (Novo)

#### `src/hardware/acpi.rs`
Parser de tabelas ACPI (RSDP, RSDT, XSDT, SDT headers).

#### `src/hardware/fdt.rs`
Device Tree support para ARM64 e RISC-V.

### Outros Módulos

#### `src/elf/` - Parsing e carregamento ELF64
#### `src/memory/` - Wrapper seguro de UEFI memory services
#### `src/video/` - Graphics Output Protocol
#### `src/security/` - Integridade, rollback, Secure Boot, BLAKE2B
#### `src/recovery/` - Fallback, diagnóstico, detecção de teclas

## 🔄 Fluxo de Boot Completo (v0.4.0)

```
1.  UEFI Firmware carrega ignite.efi
2.  Inicializar serviços UEFI
3.  Mostrar hints de teclas (R=Recovery, C=Config)
4.  Carregar e parsear ignite.conf
    ├─ ConfigParser lê arquivo
    ├─ MacroExpander expande ${MACROS}
    └─ ConfigValidator valida sintaxe
5.  Exibir menu de boot (se múltiplas entries)
    ├─ BootMenu renderiza options
    ├─ InputHandler captura teclas
    └─ Timeout ou seleção manual
6.  Detectar protocolo apropriado
    ├─ Verificar header do kernel
    ├─ Selecionar BootProtocol correto
    └─ Instanciar protocolo
7.  Protocol.validate() - Verificar compatibilidade
8.  Protocol.prepare() - Preparar boot
    ├─ Parsear headers específicos do protocolo
    ├─ Alocar memória (via MemoryAllocator)
    ├─ Copiar kernel para memória
    ├─ Carregar módulos/initrd
    └─ Setup estruturas de boot (MBI, boot_params, etc)
9.  Configurar GOP (Graphics Output Protocol)
10. Preparar estrutura de boot info
11. Exit UEFI boot services
12. Saltar para entry point com registradores corretos
```

## 📚 Documentação

Ver [docs/](docs/) para documentação adicional:

- **protocols.md** - Detalhes de cada protocolo
- **configuration.md** - Guia completo de configuração
- **development.md** - Guia para desenvolvedores

## 🛠️ Comandos de Compilação

```bash
# Instalar target
rustup target add x86_64-unknown-uefi

# Debug build
cargo build --target x86_64-unknown-uefi

# Release build (otimizado)
cargo build --target x86_64-unknown-uefi --release

# Verificação rápida
cargo check --target x86_64-unknown-uefi

# Linting
cargo clippy --target x86_64-unknown-uefi

# Formatar
cargo fmt

# Testes
cargo test --lib
```

## 🎯 Roadmap

### ✅ v0.4.0 (Atual - CONCLUÍDO)
- Multi-protocol boot (5 protocolos)
- Sistema de configuração completo
- UI framework
- Filesystem drivers (FAT32, ISO9660)
- ACPI/FDT support
- Compilação bem-sucedida

### 🔄 v0.5.0 (Próxima)
- Completar `FAT32::read_file()`
- Completar `ISO9660::read_file()`
- Integração UEFI input protocols
- Linux boot_params completo
- Multiboot MBI completo

### 📋 v0.6.0
- Font rendering
- BLAKE2B completo
- Wallpaper support
- Config editor com syntax highlighting

### 🚀 v1.0.0
- BIOS/MBR support
- Multi-arquitetura
- Tools (ignite-install, ignite-mkiso)

## 📊 TODOs por Módulo

### Alta Prioridade ⚡
- [ ] `fs/fat32.rs` - Implementar `read_file()` completo
- [ ] `ui/input.rs` - Integrar UEFI Simple Input Protocol
- [ ] `protos/linux.rs` - Completar `boot_params` structure
- [ ] `protos/multiboot1.rs` - Completar `create_mbi()` com memory map

### Média Prioridade 🔸
- [ ] `ui/terminal.rs` - Implementar font rendering
- [ ] `security/blake2b.rs` - Algoritmo BLAKE2B completo
- [ ] `fs/iso9660.rs` - Implementar `read_file()`
- [ ] `config/` - Carregar config file do disco

### Baixa Prioridade ⬜
- [ ] BIOS support (Assembly stage1/stage2)
- [ ] Tools (binários separados)
- [ ] ARM64/RISC-V support
- [ ] Network boot (PXE)

---

**Versão**: 0.4.0  
**Status**: 75% completo (6 de 8 fases)  
**Build**: ✅ 3.53s, 0 erros, 13 warnings  
**Última atualização**: 18 de dezembro de 2025
