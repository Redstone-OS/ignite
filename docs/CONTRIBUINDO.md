# Guia de Contribuição - Ignite Bootloader

Obrigado por considerar contribuir com o Ignite! Este documento fornece diretrizes para contribuir com o projeto.

## 📋 Índice

- [Código de Conduta](#código-de-conduta)
- [Como Contribuir](#como-contribuir)
- [Padrões de Código](#padrões-de-código)
- [Processo de Pull Request](#processo-de-pull-request)
- [Desenvolvimento Local](#desenvolvimento-local)
- [Áreas Priori tárias](#áreas-prioritárias)

---

## Código de Conduta

### Nossa Promessa

Estamos comprometidos em tornar a participação neste projeto uma experiência livre de assédio para todos.

### Padrões

**Comportamento esperado**:
- Usar linguagem acolhedora e inclusiva
- Respeitar pontos de vista e experiências diferentes
- Aceitar críticas construtivas graciosamente
- Focar no que é melhor para a comunidade

**Comportamento inaceitável**:
- Uso de linguagem ou imagens sexualizadas
- Comentários insultuosos/depreciativos
- Assédio público ou privado
- Publicar informações privadas sem permissão

---

## Como Contribuir

### Reportar Bugs

Antes de criar um issue:
1. Verifique se já não existe issue similar
2. Use o template de bug report
3. Inclua informações detalhadas

**Template de Bug Report**:
```markdown
**Descrição do Bug**
Descrição clara e concisa do problema.

**Passos para Reproduzir**
1. Configurar '...'
2. Executar '...'
3. Observe o erro

**Comportamento Esperado**
O que deveria acontecer.

**Comportamento Atual**
O que está acontecendo.

** Ambiente**
- Versão do Ignite: [ex: 0.1.0]
- Hardware / QEMU: [ex: QEMU 7.2]
- Firmware UEFI: [ex: OVMF, versão X]

**Logs**
```
Copiar logs serial aqui
```

**Screenshots**
Se aplicável, adicione screenshots.
```

---

### Sugerir Features

Antes de sugerir:
1. Verificar roadmap do projeto
2. Pesquisar issues existentes
3. Considerar se alinha com objetivos do projeto

**Template de Feature Request**:
```markdown
**Problema a Resolver**
Descrição clara do problema que a feature resolve.

**Solução Proposta**
Descrição da solução desejada.

**Alternativas Consideradas**
Outras abordagens que você considerou.

**Contexto Adicional**
Qualquer outro contexto relevante.
```

---

### Contribuir com Código

#### Fork e Clone

```bash
# Fork no GitHub (botão Fork)

# Clone seu fork
git clone https://github.com/SEU_USUARIO/ignite.git
cd ignite

# Adicionar repositório upstream
git remote add upstream https://github.com/redstone-os/ignite.git

# Verificar remotes
git remote -v
```

#### Criar Branch

```bash
# Atualizar main
git checkout main
git pull upstream main

# Criar branch para feature/fix
git checkout -b feature/nome-descritivo
# Exemplos:
# git checkout -b feature/multiboot2-support
# git checkout -b fix/serial-output-bug
# git checkout -b docs/improve-api-reference
```

#### Fazer Mudanças

```bash
# Fazer alterações no código

# Formatar
cargo fmt

# Lint
cargo clippy --target x86_64-unknown-uefi

# Build
cargo build --target x86_64-unknown-uefi

# Testar
cargo test --lib
```

#### Commit

Seguir [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git add .
git commit -m "tipo(escopo): descrição curta

Descrição detalhada opcional do que foi feito e por quê.

Refs: #123"
```

**Tipos de Commit**:
- `feat`: Nova funcionalidade
- `fix`: Correção de bug
- `docs`: Documentação
- `style`: Formatação (sem mudança de código)
- `refactor`: Refatoração
- `perf`: Melhoria de performance
- `test`: Adição de testes
- `chore`: Manutenção

**Exemplos**:
```bash
git commit -m "feat(protos): add Multiboot2 protocol support

Implement Multiboot2 boot protocol with tag parsing and MBI construction.

Refs: #42"

git commit -m "fix(memory): correct page table alignment check

Previous check was using wrong constant. Now validates alignment correctly.

Fixes: #89"

git commit -m "docs(api): document FrameAllocator trait

Add comprehensive documentation with examples."
```

#### Push e Pull Request

```bash
# Push para seu fork
git push origin feature/nome-descritivo

# Abrir PR no GitHub
# Use o template de PR automático
```

---

## Padrões de Código

### Rust Style Guide

Seguir [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).

#### Formatação

```bash
# Sempre usar rustfmt antes de commit
cargo fmt --all

# Verificar formatação
cargo fmt --all -- --check
```

Configuração em `rustfmt.toml`:
```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Max"
```

---

#### Naming Conventions

```rust
// Structs, Enums, Traits: PascalCase
pub struct BootConfig { }
pub enum Protocol { }
pub trait FileSystem { }

// Funções, variáveis: snake_case
pub fn load_kernel() { }
let kernel_data = vec![];

// Constantes: SCREAMING_SNAKE_CASE
pub const MAX_ENTRIES: usize = 16;
static SYSTEM_TABLE: Once<SystemTable> = Once::new();

// Lifetime parameters: `'name` (lowercase)
fn foo<'a>(x: &'a str) { }

// Type parameters: Single uppercase letter ou PascalCase
fn allocate<T>(count: usize) -> T
fn map<Allocator: FrameAllocator>(alloc: &mut Allocator)
```

---

#### Documentação

```rust
/// Carrega a configuração do bootloader.
///
/// # Argumentos
///
/// * `fs` - Sistema de arquivos
///
/// # Retorna
///
/// * `Ok(BootConfig)` - Configuração carregada
/// * `Err(BootError)` - Erro ao carregar
///
/// # Exemplos
///
/// ```no_run
/// let config = load_configuration(&mut fs)?;
/// ```
///
/// # Panics
///
/// Esta função não entra em pânico.
pub fn load_configuration(fs: &mut dyn FileSystem) -> Result<BootConfig> {
    // ...
}
```

**Seções obrigatórias**:
- Descrição resumida (primeira linha)
- Descrição detalhada (opcional)
- `# Argumentos` (se houver)
- `# Retorna` (se não for `()`)
- `# Examples` (quando útil)
- `# Panics` (se puder entrar em pânico)
- `# Safety` (se for unsafe)

---

#### Error Handling

```rust
// BOM: Usar Result
pub fn load_file(path: &str) -> Result<Vec<u8>> {
    let file = fs::open(path)?;
    let data = fs::read(file)?;
    Ok(data)
}

// RUIM: panic! em bibliotecas
pub fn load_file(path: &str) -> Vec<u8> {
    let file = fs::open(path).expect("file not found");  // ❌
    // ...
}

// OK: panic! apenas em main.rs para erros irrecuperáveis
fn main() -> ! {
    let config = load_configuration(&mut fs)
        .expect("FATAL: Não foi possível carregar configuração");
    // ...
}
```

---

#### Unsafe Code

```rust
// Sempre documentar SAFETY
/// # Safety
/// Este código assume que:
/// - `ptr` aponta para memória válida e alinhada
/// - `ptr` não é acessado concorrentemente
/// - O caller mantém ownership da memória apontada
unsafe fn write_register(ptr: *mut u32, value: u32) {
    core::ptr::write_volatile(ptr, value);
}

// Minimizar escopo de unsafe
fn foo() {
    let x = 42;
    let result = unsafe {
        // Apenas o mínimo necessário dentro de unsafe
        some_unsafe_function(x)
    };
    // Código seguro continua aqui
}
```

---

### Clippy

```bash
# Executar clippy
cargo clippy --target x86_64-unknown-uefi -- -D warnings

# Permitir lints específicos (quando justificado)
#[allow(clippy::too_many_arguments)]  // Justificado em FFI
fn uefi_function(a: u64, b: u64, c: u64, d: u64, e: u64, f: u64) { }
```

---

## Processo de Pull Request

### Checklist Pré-PR

- [ ] Código compila sem erros
- [ ] Código compila sem warnings
- [ ] `cargo fmt` executado
- [ ] `cargo clippy` passa
- [ ] Documentação inline atualizada
- [ ] Documentação externa atualizada (se aplicável)
- [ ] Commits seguem Conventional Commits
- [ ] Brand está atualizada com upstream/main

---

### Após Abrir PR

1. **CI passará automaticamente**:
   - Formatação
   - Clippy
   - Build (debug e release)

2. **Code Review**:
   - Mantenedores revisarão o código
   - Responda aos comentários
   - Faça mudanças solicitadas

3. **Aprovar e Merge**:
   - Após aprovação, PR será merged
   - Pode ser squashed em um único commit

---

### Template de PR

```markdown
## Descrição

Breve descrição do que esta PR faz.

## Tipo de Mudança

- [ ] Bug fix (mudança que corrige um issue)
- [ ] Nova feature (mudança que adiciona funcionalidade)
- [ ] Breaking change (fix ou feature que quebraria funcionalidade existente)
- [ ] Documentação

## Checklist

- [ ] Código segue style guide
- [ ] Documentação inline atualizada
- [ ] Testes adicionados/atualizados (se aplicável)
- [ ] Documentação externa atualizada (se aplicável)

## Issues Relacionadas

Closes #123
Refs #456

## Screenshots (se aplicável)

## Notas para Reviewers

Informações adicionais para facilitar review.
```

---

## Desenvolvimento Local

### Setup Completo

Ver `docs/DESENVOLVIMENTO.md` para setup detalhado.

**Resumo**:
```bash
# Rust nightly + target
rustup toolchain install nightly
rustup target add x86_64-unknown-uefi --toolchain nightly

# Build tools
sudo apt install build-essential git qemu-system-x86 ovmf

# Clone e build
git clone https://github.com/redstone-os/ignite.git
cd ignite
cargo build --release --target x86_64-unknown-uefi
```

---

### Workflow Recomendado

```bash
# 1. Atualizar branch
git checkout main
git pull upstream main

# 2. Criar feature branch
git checkout -b feature/minha-feature

# 3. Ciclo de desenvolvimento
while true; do
    # Editar código
    vim src/...
    
    # Verificar
    cargo fmt
    cargo clippy --target x86_64-unknown-uefi
    
    # compilar
    cargo build --target x86_64-unknown-uefi
    
    # Testar em QEMU
    ./tools/run_qemu.sh
done

# 4. Commit
git add .
git commit -m "feat(modulo): descrição"

# 5. Push e PR
git push origin feature/minha-feature
```

---

## Áreas Prioritárias

Procurando por onde começar? Aqui estão áreas que precisam de ajuda:

### 🔴 Alta Prioridade

- **Linux Boot Protocol**: Completar implementação de bzImage
- **Multiboot2**: Finalizar suporte completo com todas as tags
- **Testes**: Expandir cobertura de testes

### 🟡 Média Prioridade

- **AArch64 Port**: Início do suporte ARM64
- **RISC-V Port**: Início do suporte RISC-V
- **Network Boot**: PXE/HTTP boot
- **Documentação**: Tradução para inglês

### 🟢 Baixa Prioridade / Good First Issue

- **UI Enhancements**: Melhorias visuais no menu
- **Wallpaper Support**: Suporte a PNG/JPEG
- **Config Validator**: Ferramenta CLI para validar ignite.conf
- **Examples**: Mais exemplos de configuração

**Issues marcadas com `good-first-issue`**: Perfeitas para iniciantes!

---

## Comunicação

- **GitHub Issues**: Bugs, features, perguntas
- **GitHub Discussions**: Discussões gerais, ideias
- **Discord**: Chat em tempo real (link no README)

---

## Reconhecimento

Contribuidores serão creditados em:
- `README.md`
- Release notes
- `CONTRIBUTORS.md`

---

## Licença

Ao contribuir, você concorda que suas contribuições serão licenciadas sob a mesma licença do projeto (MIT).

---

**Obrigado por contribuir com o Ignite! 🦀🔥**

---

**Última Atualização**: 2025-12-21
