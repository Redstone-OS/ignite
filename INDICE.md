# Índice - Ignite (Bootloader UEFI)

Este diretório contém o bootloader UEFI do Redstone OS, responsável por inicializar o sistema e carregar o kernel.

## Estrutura de Arquivos

```bash
ignite/
├── .clippy.toml              # Configuração do Clippy (linter)
├── .editorconfig             # Configuração de editor
├── .gitignore                # Arquivos ignorados pelo Git
├── AUTHORS.md                # Autores e contribuidores
├── CHANGELOG.md              # Histórico de mudanças
├── CODE_OF_CONDUCT.md        # Código de conduta
├── CONTRIBUTING.md           # Guia de contribuição
├── Cargo.toml                # Configuração do pacote Rust
├── INDICE.md                 # Este arquivo
├── LICENSE                   # Licença MIT
├── README.md                 # Documentação principal
├── SECURITY.md               # Política de segurança
├── rustfmt.toml              # Configuração de formatação
├── docs/                     # Documentação adicional
│   └── README.md             # Recursos e referências
└── src/                      # Código-fonte
    ├── main.rs               # Entry point (11 linhas)
    ├── lib.rs                # Biblioteca principal (orquestração)
    ├── error.rs              # Sistema de erros (175 linhas)
    ├── types.rs              # Tipos compartilhados (68 linhas)
    ├── memory/               # Gerenciamento de memória
    │   ├── mod.rs            # Módulo público
    │   └── allocator.rs      # Alocador UEFI (86 linhas)
    ├── video/                # Configuração de vídeo
    │   ├── mod.rs            # Módulo público + trait
    │   └── gop.rs            # Graphics Output Protocol (73 linhas)
    ├── fs/                   # Sistema de arquivos
    │   ├── mod.rs            # Módulo público
    │   ├── loader.rs         # Carregador de arquivos (93 linhas)
    │   └── initfs.rs         # Carregador de InitFS (25 linhas)
    ├── elf/                  # Parsing e carregamento ELF
    │   ├── mod.rs            # Módulo público
    │   ├── parser.rs         # Parser ELF (56 linhas)
    │   └── loader.rs         # Carregador de segmentos (88 linhas)
    ├── recovery/             # Sistema de recuperação [NOVO]
    │   ├── mod.rs            # Módulo público
    │   ├── fallback.rs       # Sistema de fallback (118 linhas)
    │   ├── keydetect.rs      # Detecção de teclas (28 linhas)
    │   └── diagnostics.rs    # Diagnóstico (56 linhas)
    ├── security/             # Segurança [NOVO - Em desenvolvimento]
    │   ├── mod.rs            # Módulo público
    │   ├── integrity.rs      # Verificação de integridade
    │   ├── rollback.rs       # Proteção contra rollback
    │   └── secureboot.rs     # Suporte a Secure Boot
    ├── config/               # Configuração [NOVO - Em desenvolvimento]
    │   ├── mod.rs            # Módulo público
    │   └── boot_config.rs    # Configuração de boot e multi-boot
    └── ui/                   # Interface de usuário [NOVO - Em desenvolvimento]
        ├── mod.rs            # Módulo público
        └── boot_menu.rs      # Menu de boot interativo
```

## Visão Geral do Projeto

O Ignite é um bootloader UEFI moderno desenvolvido em Rust, com arquitetura modular profissional. A versão 0.3.0 representa uma evolução significativa com sistema de recuperação, segurança e multi-boot.

### Estatísticas

- **Total de arquivos**: 33 (código + documentação)
- **Linhas de código**: ~1200 linhas
- **Módulos**: 9 especializados
- **Cobertura de documentação**: 100%
- **Versão**: 0.3.0

## Descrição dos Componentes

### 📋 Documentação

#### README.md
Documentação principal com visão geral completa, arquitetura, instruções de compilação, roadmap e status de cada fase.

#### CONTRIBUTING.md
Guia completo de contribuição incluindo:
- Como reportar bugs e sugerir melhorias
- Processo de Pull Request
- Convenções de código e commits
- Estrutura do projeto

#### CHANGELOG.md
Histórico de mudanças seguindo Keep a Changelog:
- v0.3.0: Fases 2, 3 e 4 (estrutura básica)
- v0.2.0: Refatoração modular completa
- v0.1.0: Implementação inicial

#### CODE_OF_CONDUCT.md
Código de conduta baseado no Contributor Covenant v2.1

#### SECURITY.md
Política de segurança com processo de divulgação responsável e áreas críticas

#### AUTHORS.md
Lista de autores, contribuidores e agradecimentos

#### LICENSE
Licença MIT do projeto

### ⚙️ Configuração

#### .gitignore
Ignora arquivos de build, temporários e específicos de IDE

#### .editorconfig
Configuração de editor para consistência de código entre diferentes editores

#### rustfmt.toml
Configuração de formatação de código Rust

#### .clippy.toml
Configuração de linting com regras específicas para código de sistema

### 💻 Código-Fonte

#### src/main.rs (11 linhas)
Entry point minimalista que apenas chama `ignite::boot()`

#### src/lib.rs
Biblioteca principal que orquestra todo o processo de boot em 6 etapas

#### src/error.rs (175 linhas)
Sistema de erros robusto com tipos específicos para cada categoria

#### src/types.rs (68 linhas)
Tipos compartilhados: KernelArgs, Framebuffer, LoadedFile, LoadedKernel

#### src/memory/ (Módulo de Memória)
Gerenciamento de memória UEFI com wrapper seguro

#### src/video/ (Módulo de Vídeo)
Abstração de vídeo via trait VideoOutput e implementação GOP

#### src/fs/ (Sistema de Arquivos)
Carregamento de arquivos UEFI e InitFS opcional

#### src/elf/ (Módulo ELF)
Parsing e carregamento de arquivos ELF com validação

#### src/recovery/ (Módulo de Recuperação) [NOVO]
Sistema de fallback, diagnóstico e detecção de teclas especiais

#### src/security/ (Módulo de Segurança) [NOVO - Em desenvolvimento]
Verificação de integridade, proteção contra rollback e Secure Boot

#### src/config/ (Módulo de Configuração) [NOVO - Em desenvolvimento]
Configuração de boot via arquivo e suporte a multi-boot

#### src/ui/ (Interface de Usuário) [NOVO - Em desenvolvimento]
Menu de boot interativo para seleção de sistema operacional

## Fluxo de Boot

```
1. UEFI Firmware carrega ignite.efi
2. main.rs chama ignite::boot()
3. Inicializa serviços UEFI
4. Mostra hints de teclas (R=Recovery, C=Config)
5. Executa diagnóstico básico
6. Seleciona kernel (com fallback)
7. FileLoader carrega kernel "forge"
8. ElfParser valida e parseia ELF
9. ElfLoader aloca memória e copia segmentos
10. GopVideoOutput configura framebuffer
11. InitFsLoader carrega sistema de arquivos inicial
12. Prepara KernelArgs com todas as informações
13. Exit boot services
14. Salta para entry point do kernel
```

## Compilação

```bash
# Instalar target
rustup target add x86_64-unknown-uefi

# Compilar
cargo build --target x86_64-unknown-uefi --release

# Verificar código
cargo clippy --target x86_64-unknown-uefi
cargo fmt --check

# Output
target/x86_64-unknown-uefi/release/ignite.efi
```

## Roadmap

### ✅ Fase 1: Fundação (Concluída)
- Modularização completa
- Sistema de erros robusto
- Documentação profissional

### ✅ Fase 2: Confiabilidade (Básico Concluído)
- Sistema de fallback
- Diagnóstico não-bloqueante
- Hint de tecla R

### 🔄 Fase 3: Segurança (Estrutura Criada)
- Verificação de integridade
- Proteção contra rollback
- Preparação para Secure Boot

### 🔄 Fase 4: Funcionalidades (Estrutura Criada)
- Menu de boot configurável
- Sistema de configuração
- Multi-boot (Redstone/Linux/Windows)

### 📋 Fase 5: Otimização (Futuro)
- Performance
- Testes completos
- Release 1.0

## TODOs Principais

### Alta Prioridade
1. Implementar persistência de contador de boot (variáveis UEFI)
2. Implementar detecção de tecla R para recovery
3. Implementar SHA-256 real para verificação de integridade
4. Implementar parser de arquivo de configuração (.cfg/.ini)

### Média Prioridade
5. Implementar shell de recuperação interativo
6. Implementar menu de boot interativo
7. Implementar detecção automática de Linux/Windows
8. Implementar extração de versão de kernel

### Baixa Prioridade
9. Implementar validação de assinaturas digitais
10. Implementar detecção de Secure Boot
11. Otimizações de performance

---

**Versão**: 0.3.0  
**Status**: Fases 1-2 concluídas, Fases 3-4 em desenvolvimento  
**Última atualização**: 15 de dezembro de 2025
