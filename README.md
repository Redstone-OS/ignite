# Ignite - Bootloader UEFI para Redstone OS

**Versão**: 0.3.0  
**Linguagem**: Rust  
**Arquitetura**: x86_64  
**Tipo**: Bootloader UEFI  
**Status**: Desenvolvimento Ativo

## Visão Geral

Ignite é um bootloader UEFI moderno desenvolvido em Rust para o sistema operacional Redstone. Ele é responsável por carregar o kernel do sistema, configurar o ambiente de hardware e transferir o controle para o kernel.

### Características Principais

- ✅ **Escrito em Rust** - Segurança de memória garantida em tempo de compilação
- ✅ **Arquitetura Modular** - Código organizado em módulos especializados
- ✅ **Parsing ELF Robusto** - Suporte completo a arquivos ELF64
- ✅ **Configuração de Vídeo** - Inicialização automática de framebuffer via GOP
- ✅ **Suporte a InitFS** - Carregamento opcional de sistema de arquivos inicial
- ✅ **Tratamento de Erros** - Sistema de erros centralizado e tipado
- ✅ **Sistema de Fallback** - Recuperação automática de falhas
- ✅ **Modo de Recuperação** - Shell de recuperação interativo (em desenvolvimento)
- 🔄 **Verificação de Integridade** - SHA-256 e proteção contra rollback (em desenvolvimento)
- 🔄 **Menu de Boot** - Seleção de sistema operacional (em desenvolvimento)
- 🔄 **Multi-Boot** - Suporte a Linux e Windows (em desenvolvimento)

## Arquitetura

### Estrutura de Módulos

```
src/
├── main.rs              # Entry point (11 linhas)
├── lib.rs               # Biblioteca principal e orquestração
├── error.rs             # Sistema de erros centralizado
├── types.rs             # Tipos compartilhados (KernelArgs, Framebuffer, etc)
├── memory/              # Gerenciamento de memória
│   ├── mod.rs
│   └── allocator.rs     # Wrapper de alocação UEFI
├── video/               # Configuração de vídeo
│   ├── mod.rs
│   └── gop.rs           # Graphics Output Protocol
├── fs/                  # Sistema de arquivos
│   ├── mod.rs
│   ├── loader.rs        # Carregador de arquivos UEFI
│   └── initfs.rs        # Carregador de InitFS
├── elf/                 # Parsing e carregamento de ELF
│   ├── mod.rs
│   ├── parser.rs        # Parser de arquivos ELF
│   └── loader.rs        # Carregador de segmentos ELF
├── recovery/            # Sistema de recuperação
│   ├── mod.rs
│   ├── fallback.rs      # Sistema de fallback
│   ├── keydetect.rs     # Detecção de teclas especiais
│   └── diagnostics.rs   # Diagnóstico de sistema
├── security/            # Segurança (em desenvolvimento)
│   ├── mod.rs
│   ├── integrity.rs     # Verificação de integridade
│   ├── rollback.rs      # Proteção contra rollback
│   └── secureboot.rs    # Suporte a Secure Boot
├── config/              # Configuração (em desenvolvimento)
│   ├── mod.rs
│   └── boot_config.rs   # Configuração de boot e multi-boot
└── ui/                  # Interface de usuário (em desenvolvimento)
    ├── mod.rs
    └── boot_menu.rs     # Menu de boot interativo
```

### Fluxo de Boot

```
1. UEFI Firmware carrega ignite.efi
2. Inicializa Serviços UEFI
3. Mostra hints de teclas especiais (R=Recovery, C=Config)
4. Executa diagnóstico básico do sistema
5. Seleciona kernel (com fallback se necessário)
6. Carrega kernel "forge"
7. Parseia e valida ELF
8. Aloca memória contígua
9. Copia segmentos PT_LOAD
10. Configura GOP (Graphics Output Protocol)
11. Carrega InitFS opcional
12. Prepara KernelArgs
13. Exit Boot Services
14. Salta para entry point do kernel
```

## Compilação

### Pré-requisitos

- Rust (edição 2024)
- Target `x86_64-unknown-uefi`

### Instalar Target

```bash
rustup target add x86_64-unknown-uefi
```

### Compilar

```bash
# Debug
cargo build --target x86_64-unknown-uefi

# Release (recomendado)
cargo build --target x86_64-unknown-uefi --release
```

### Saída

O arquivo compilado estará em:
```
target/x86_64-unknown-uefi/release/ignite.efi
```

## Uso

### Estrutura de Boot

O bootloader espera encontrar os seguintes arquivos no mesmo volume:

```
/
├── ignite.efi        # Bootloader
├── forge             # Kernel (ELF64)
└── initfs            # Sistema de arquivos inicial (opcional)
```

### Configuração (Futuro)

O bootloader poderá ser configurado via arquivo `boot.cfg` ou `ignite.ini`:

```ini
[boot]
menu_enabled = false      # Menu desabilitado por padrão
default_os = redstone     # OS padrão
timeout = 5               # Timeout em segundos

[os.redstone]
name = "Redstone OS"
kernel = "forge"
initfs = "initfs"

[os.linux]
name = "Linux"
kernel = "vmlinuz"
initrd = "initrd.img"

[os.windows]
name = "Windows"
efi = "\\EFI\\Microsoft\\Boot\\bootmgfw.efi"
```

### Teclas Especiais

- **R** - Entra em modo de recuperação (quando implementado)
- **C** - Abre configuração (quando implementado)

### Argumentos Passados ao Kernel

O bootloader passa uma estrutura `KernelArgs` para o kernel contendo:

| Campo | Descrição |
|-------|-----------|
| `kernel_base` | Endereço base do kernel na memória |
| `kernel_size` | Tamanho do kernel em bytes |
| `stack_base` | Endereço base da stack (0 = kernel configura) |
| `stack_size` | Tamanho da stack |
| `env_base` | Endereço das variáveis de ambiente |
| `env_size` | Tamanho das variáveis de ambiente |
| `hwdesc_base` | Endereço da descrição de hardware (ACPI) |
| `hwdesc_size` | Tamanho da descrição de hardware |
| `areas_base` | Endereço do mapa de memória |
| `areas_size` | Tamanho do mapa de memória |
| `bootstrap_base` | Endereço do InitFS |
| `bootstrap_size` | Tamanho do InitFS |

## Dependências

| Crate | Versão | Propósito |
|-------|--------|-----------|
| `uefi` | 0.28.0 | Biblioteca UEFI para Rust |
| `uefi-services` | 0.25.0 | Serviços auxiliares UEFI |
| `log` | 0.4 | Sistema de logging |
| `goblin` | 0.8 | Parser de formatos binários (ELF) |

## Roadmap

### ✅ Fase 1: Fundação (Concluída)

- [x] Modularização do código
- [x] Sistema de erros centralizado
- [x] Abstração de hardware
- [x] Compilação bem-sucedida
- [x] Documentação completa

### ✅ Fase 2: Confiabilidade (Básico Concluído)

- [x] Estrutura de fallback
- [x] Diagnóstico não-bloqueante
- [x] Hint de tecla R para recovery
- [ ] Persistência de contador de boot (TODO)
- [ ] Detecção de tecla R (TODO)
- [ ] Shell de recuperação interativo (TODO)

### 🔄 Fase 3: Segurança (Estrutura Criada)

- [x] Estrutura de módulos de segurança
- [ ] Verificação de integridade SHA-256 (TODO)
- [ ] Proteção contra rollback (TODO)
- [ ] Detecção de Secure Boot (TODO)
- [ ] Validação de assinaturas (TODO)

### 🔄 Fase 4: Funcionalidades (Estrutura Criada)

- [x] Estrutura de configuração
- [x] Estrutura de menu de boot
- [x] Suporte a multi-boot (Redstone/Linux/Windows)
- [ ] Parser de arquivo de configuração (TODO)
- [ ] Menu interativo (TODO)
- [ ] Detecção automática de OS (TODO)
- [ ] Tecla C para configuração (TODO)

### 📋 Fase 5: Otimização (Futuro)

- [ ] Performance
- [ ] Testes completos
- [ ] Release 1.0

## Desenvolvimento

### Estrutura de Erros

O bootloader usa um sistema de erros tipado e centralizado:

```rust
pub enum BootError {
    FileSystem(FileSystemError),
    Elf(ElfError),
    Memory(MemoryError),
    Video(VideoError),
    Config(ConfigError),
    Generic(&'static str),
}
```

### Sistema de Fallback

O bootloader tenta até 3 vezes antes de entrar em modo de recuperação (estilo Windows):

```rust
pub struct BootOptions {
    pub primary_kernel: KernelEntry,
    pub recovery_kernel: Option<KernelEntry>,
    pub boot_attempts: u8,
    pub max_attempts: u8, // Padrão: 3
}
```

### Logging

O bootloader usa a crate `log` para logging. As mensagens são enviadas para a saída serial UEFI.

Exemplo de saída:

```
═══════════════════════════════════════════════════
  Bootloader Ignite v0.3.0 - Redstone OS
═══════════════════════════════════════════════════
Pressione 'R' para modo de recuperação
Etapa 1/6: Diagnóstico do sistema...
✓ forge encontrado (524288 bytes)
○ initfs não encontrado (opcional)
Etapa 2/6: Carregando kernel...
Kernel selecionado: Redstone OS
...
```

## Contribuindo

Este bootloader faz parte do projeto Redstone OS. Para contribuir:

1. Siga os padrões de código Rust
2. Mantenha a modularidade
3. Adicione testes quando possível
4. Documente mudanças significativas
5. Veja `CONTRIBUTING.md` para mais detalhes

## Licença

MIT License - Veja `LICENSE` para detalhes

## Segurança

Para reportar vulnerabilidades de segurança, veja `SECURITY.md`

---

**Última atualização**: 15 de dezembro de 2025  
**Status**: v0.3.0 - Fases 1-2 concluídas, Fases 3-4 em desenvolvimento  
**Próxima versão**: v0.4.0 - Completar Fase 2 e 3
