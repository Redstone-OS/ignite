# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [Não Lançado]

### 🎯 Planejado
- [ ] Suporte a RISC-V 64-bit
- [ ] Suporte a AArch64 (ARM64)
- [ ] Network Boot (PXE/HTTP)
- [ ] Criptografia de disco (LUKS)
- [ ] GUI avançada com mouse

---

## [0.1.0] - 2025-12-21

### 🎉 Lançamento Inicial

Primeira versão pública do Ignite Bootloader - um bootloader UEFI industrial desenvolvido em Rust.

### ✨ Adicionado

#### Core Features
- ✅ **Bootloader UEFI completo** em Rust `no_std`
- ✅ **Implementação FFI UEFI pura** sem dependências externas pesadas
- ✅ **Suporte a x86_64** (arquitetura principal)

#### Protocolos de Boot
- ✅ **Redstone/Limine Protocol** (nativo, ELF64)
  - Higher-half kernel mapping
  - BootInfo completo
  - Framebuffer GOP
  - Memory map detalhado
- ✅ **Linux Boot Protocol** (bzImage)
  - boot_params structure
  - initrd support
- ✅ **Multiboot2**
  - MBI completo
  - Tag parsing
  - Module loading
- ✅ **UEFI Chainload**
  - LoadImage/StartImage
  - Fallback para outros bootloaders

#### Gerenciamento de Memória
- ✅ **BumpAllocator** otimizado para boot
- ✅ **x86_64 Paging** (4 níveis)
  - Identity mapping
  - Higher-half kernel mapping
  - Huge pages (2MB)
  - NX bit support
- ✅ **FrameAllocator** trait abstração
- ✅ **Memory map** handoff para kernel

#### Sistema de Arquivos
- ✅ **FAT32** driver nativo (leitura)
- ✅ **UEFI Simple File System** integration
- ✅ **Virtual File System (VFS)** abstração
- ✅ **Path resolution** (`boot():/`, `root():/`)

#### Configuração
- ✅ **Parser customizado** para `ignite.conf`
- ✅ **Múltiplas entradas de boot**
- ✅ **Timeout configurável**
- ✅ **Default entry**
- ✅ **Resolução de vídeo customizável**
- ✅ **Command line arguments** para kernels

#### Interface de Usuário
- ✅ **Menu gráfico interativo** com GOP
- ✅ **Font rendering** bitmap
- ✅ **Temas visuais** configuráveis
- ✅ **Navegação por teclado** (↑↓ Enter ESC)
- ✅ **Timeout visual** com countdown

#### Segurança
- ✅ **Secure Boot detection**
- ✅ **TPM 2.0 measurements** (PCR 9)
- ✅ **Políticas de segurança** configuráveis
- ✅ **Validação de binários**
- ✅ **Input validation** completa
- ✅ **Memory safety** (Rust)
- ✅ **Minimal unsafe code**

#### ELF Support
- ✅ **ELF64 parser** completo
- ✅ **Program header** loading
- ✅ **Section header** parsing
- ✅ **Relocation support** (básico)
- ✅ **Validação de magic bytes**

#### Diagnósticos e Recovery
- ✅ **Recovery mode** automático
- ✅ **Health checks** de entradas
- ✅ **Diagnósticos de configuração**
- ✅ **Fallback automático** em falhas

#### Documentação
- ✅ **README.md** completo em português
- ✅ **12 documentos técnicos** detalhados:
  - ARQUITETURA.md (1.050 linhas)
  - DESENVOLVIMENTO.md (798 linhas)
  - CONFIGURACAO.md (930 linhas)
  - BUILD.md (820 linhas)
  - API.md (510 linhas)
  - TROUBLESHOOTING.md (510 linhas)
  - CONTRIBUINDO.md (610 linhas)
  - SEGURANCA.md (480 linhas)
  - PROTOCOLOS.md (510 linhas)
  - MEMORIA.md (480 linhas)
  - FILESYSTEM.md (470 linhas)
- ✅ **Diagramas Mermaid** de arquitetura
- ✅ **Exemplos práticos** em todos os guias

#### Testes
- ✅ **109 testes automatizados**:
  - 20 testes de configuração
  - 18 testes de memória
  - 17 testes de ELF
  - 12 testes de segurança
  - 15 testes de filesystem
  - 15 testes de integração
  - 12 testes de regressão
- ✅ **~90% cobertura** de código crítico
- ✅ **Script de testes** automatizado (`run_tests.ps1`)

#### Ferramentas
- ✅ **ignite.py** - Sistema de build industrial
  - Menu interativo profissional
  - Progress bars e logs
  - Métricas históricas
  - Sistema de cache
  - Health monitoring
  - Build, test, check, distribution
- ✅ **Logging detalhado** para debugging

#### Build System
- ✅ **Perfis otimizados** (debug, release)
- ✅ **LTO habilitado** em release
- ✅ **Stripping de símbolos**
- ✅ **Otimização de tamanho** (`opt-level = "z"`)
- ✅ **Target x86_64-unknown-uefi**

### 🔧 Configuração

#### Arquivos de Configuração
- `.clippy.toml` - Regras Clippy customizadas
- `.editorconfig` - Formatação consistente
- `rustfmt.toml` - Estilo de código Rust
- `rust-toolchain.toml` - Toolchain nightly pinned

### 📊 Estatísticas

- **Linhas de código**: ~15.000 (Rust)
- **Linhas de documentação**: ~7.000 (Markdown)
- **Linhas de testes**: ~3.500
- **Módulos**: 14 principais
- **Dependências**: 4 (minimal)
- **Tamanho do binário**: ~250 KB (release)

### 🐛 Bugs Conhecidos

- ⚠️ **Multiboot2**: Algumas tags avançadas não implementadas
- ⚠️ **Linux Protocol**: bzImage muito antigos podem falhar
- ⚠️ **FAT32**: Apenas leitura (escrita não implementada)

### ⚡ Performance

- **Boot time**: < 500ms (QEMU, até menu)
- **Kernel load**: ~50-100ms (kernel 5MB)
- **Config parsing**: < 10ms

---

## [0.0.x] - Desenvolvimento Interno

### Notas

Versões 0.0.x foram desenvolvimento interno experimental antes do lançamento público.
Não há suporte para essas versões.

---

## 📝 Tipos de Mudanças

- `✨ Adicionado` - Para novas funcionalidades
- `🔧 Modificado` - Para mudanças em funcionalidades existentes
- `🗑️ Removido` - Para funcionalidades removidas
- `🐛 Corrigido` - Para correções de bugs
- `🔒 Segurança` - Para correções de vulnerabilidades
- `📚 Documentação` - Para mudanças na documentação
- `⚡ Performance` - Para melhorias de performance
- `🔨 Interno` - Para mudanças internas (refactoring, etc)

---

## 🔗 Links

- **Repositório**: https://github.com/redstone-os/ignite
- **Issues**: https://github.com/redstone-os/ignite/issues
- **Releases**: https://github.com/redstone-os/ignite/releases

---

**Última atualização**: 2025-12-21
