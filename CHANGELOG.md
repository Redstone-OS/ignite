# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [0.3.0] - 2025-12-15

### Adicionado
- **Módulo de Recuperação** (`src/recovery/`)
  - Sistema de fallback com tentativas múltiplas (estilo Windows)
  - Diagnóstico básico não-bloqueante de sistema
  - Detecção de teclas especiais (R para recovery)
  - Estrutura para shell de recuperação interativo
- **Módulo de Segurança** (`src/security/`) - Estrutura básica
  - Verificação de integridade com SHA-256 (TODO: implementar hash real)
  - Proteção contra rollback com versionamento semver
  - Preparação para Secure Boot (detecção de estado)
- **Módulo de Configuração** (`src/config/`) - Estrutura básica
  - Sistema de configuração via arquivo (.cfg/.ini)
  - Suporte a multi-boot (Redstone/Linux/Windows)
  - Estrutura de entradas de OS
- **Módulo de UI** (`src/ui/`) - Estrutura básica
  - Menu de boot interativo (TODO: implementar interface)
  - Hint de tecla C para configuração
- Hints de teclas especiais no boot (R e C)
- Diagnóstico de arquivos essenciais
- Seleção de kernel com fallback

### Modificado
- `lib.rs` atualizado com novos módulos e fluxo de boot expandido
- Fluxo de boot agora tem 6 etapas ao invés de 5
- Documentação completa atualizada

### Notas Técnicas
- Módulos `security`, `config` e `ui` estão comentados temporariamente devido a limitações de `no_std`
- Todos os TODOs estão documentados para implementação futura
- Sistema opera em modo permissivo (alerta mas não bloqueia)

## [0.2.0] - 2025-12-15

### Adicionado
- **Arquitetura Modular Completa**
  - Módulo `error` - Sistema de erros centralizado
  - Módulo `types` - Tipos compartilhados
  - Módulo `memory` - Gerenciamento de memória UEFI
  - Módulo `video` - Configuração de vídeo (GOP)
  - Módulo `fs` - Sistema de arquivos
  - Módulo `elf` - Parsing e carregamento de ELF
- **Documentação Profissional**
  - README.md completo
  - CONTRIBUTING.md
  - CODE_OF_CONDUCT.md
  - SECURITY.md
  - AUTHORS.md
  - LICENSE (MIT)
  - INDICE.md
- **Configurações de Projeto**
  - .gitignore
  - .editorconfig
  - rustfmt.toml
  - .clippy.toml
  - docs/README.md

### Modificado
- `main.rs` drasticamente simplificado (264 → 11 linhas)
- Código organizado em 13 arquivos especializados
- Sistema de erros robusto substituindo `.expect()`

### Removido
- Código monolítico de `main.rs`
- Tratamento de erros básico com `.expect()`

## [0.1.0] - 2025-12-14

### Adicionado
- Implementação inicial do bootloader
- Carregamento de kernel ELF
- Parsing de ELF com `goblin`
- Configuração de GOP (Graphics Output Protocol)
- Carregamento de InitFS opcional
- Preparação de `KernelArgs`
- Transferência de controle para kernel

### Notas
- Implementação monolítica em `main.rs` (264 linhas)
- Tratamento de erros básico com `.expect()`
- Funcional mas não modular

---

## Tipos de Mudanças

- `Adicionado` para novas funcionalidades
- `Modificado` para mudanças em funcionalidades existentes
- `Descontinuado` para funcionalidades que serão removidas
- `Removido` para funcionalidades removidas
- `Corrigido` para correções de bugs
- `Segurança` para vulnerabilidades corrigidas

## Links

- [0.3.0]: Fases 2, 3 e 4 - Sistema de recuperação e segurança
- [0.2.0]: Fase 1 - Refatoração modular completa
- [0.1.0]: Implementação inicial monolítica
