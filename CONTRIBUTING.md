# Contribuindo para o Ignite

Obrigado por considerar contribuir para o Ignite! Este documento fornece diretrizes para contribuir com o projeto.

## Código de Conduta

Ao participar deste projeto, você concorda em seguir nosso [Código de Conduta](CODE_OF_CONDUCT.md).

## Como Posso Contribuir?

### Reportando Bugs

Antes de criar um relatório de bug, verifique se o problema já foi reportado. Se você encontrar um bug:

1. **Use o GitHub Issues** para reportar
2. **Descreva o problema** claramente
3. **Forneça passos para reproduzir** o bug
4. **Inclua informações do sistema**:
   - Versão do Rust
   - Versão do Ignite
   - Hardware (se relevante)
   - Firmware UEFI

**Exemplo de relatório de bug:**

```markdown
**Descrição**: O bootloader falha ao carregar kernels maiores que 2MB

**Passos para reproduzir**:
1. Compilar kernel com tamanho > 2MB
2. Tentar boot com Ignite
3. Observar falha na alocação de memória

**Comportamento esperado**: Kernel deve carregar independente do tamanho

**Ambiente**:
- Ignite v0.2.0
- Rust 1.75.0
- QEMU 8.0
```

### Sugerindo Melhorias

Sugestões de melhorias são bem-vindas! Para sugerir:

1. **Verifique se já não foi sugerido** no GitHub Issues
2. **Descreva a melhoria** detalhadamente
3. **Explique por que seria útil**
4. **Forneça exemplos** de uso, se possível

### Pull Requests

1. **Fork o repositório** e crie um branch a partir de `main`
2. **Siga as convenções de código** (veja abaixo)
3. **Adicione testes** se aplicável
4. **Atualize a documentação** se necessário
5. **Escreva mensagens de commit claras**
6. **Certifique-se de que compila** sem erros ou warnings

#### Processo de Pull Request

1. Atualize o README.md com detalhes de mudanças, se aplicável
2. Atualize o CHANGELOG.md com suas mudanças
3. O PR será revisado por um mantenedor
4. Faça as mudanças solicitadas, se houver
5. Após aprovação, o PR será merged

## Convenções de Código

### Estilo Rust

- **Use `rustfmt`** para formatação:
  ```bash
  cargo fmt
  ```

- **Use `clippy`** para linting:
  ```bash
  cargo clippy --target x86_64-unknown-uefi
  ```

- **Siga as convenções Rust**:
  - Snake_case para funções e variáveis
  - PascalCase para tipos e traits
  - SCREAMING_SNAKE_CASE para constantes

### Documentação

- **Documente funções públicas** com `///`
- **Inclua exemplos** quando apropriado
- **Documente erros possíveis**
- **Use português** para comentários e documentação

**Exemplo:**

```rust
/// Carrega um arquivo do sistema de arquivos UEFI
///
/// # Argumentos
/// * `filename` - Nome do arquivo a carregar
///
/// # Retorna
/// Informações sobre o arquivo carregado (ponteiro e tamanho)
///
/// # Erros
/// Retorna `FileSystemError::FileNotFound` se o arquivo não existir
pub fn load_file(&mut self, filename: &'static str) -> Result<LoadedFile> {
    // ...
}
```

### Estrutura de Commits

Use mensagens de commit descritivas seguindo o padrão:

```
tipo(escopo): descrição curta

Descrição mais longa explicando o que mudou e por quê.

Fixes #123
```

**Tipos de commit:**
- `feat`: Nova funcionalidade
- `fix`: Correção de bug
- `docs`: Mudanças na documentação
- `style`: Formatação, ponto e vírgula, etc
- `refactor`: Refatoração de código
- `test`: Adição ou correção de testes
- `chore`: Tarefas de manutenção

**Exemplos:**

```
feat(elf): adicionar suporte a relocações dinâmicas

Implementa parsing e aplicação de relocações R_X86_64_RELATIVE
para suportar PIE (Position Independent Executables).

Fixes #45
```

```
fix(memory): corrigir vazamento de memória no file loader

O buffer temporário não estava sendo liberado após carregar o kernel.
Agora free_pages é chamado após a cópia dos segmentos.

Fixes #67
```

## Estrutura do Projeto

```
ignite/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Biblioteca principal
│   ├── error.rs          # Sistema de erros
│   ├── types.rs          # Tipos compartilhados
│   ├── memory/           # Gerenciamento de memória
│   ├── video/            # Configuração de vídeo
│   ├── fs/               # Sistema de arquivos
│   └── elf/              # Parsing e carregamento ELF
├── tests/                # Testes de integração (futuro)
├── docs/                 # Documentação adicional
├── Cargo.toml            # Configuração do projeto
└── README.md             # Documentação principal
```

## Desenvolvimento

### Configuração do Ambiente

1. **Instalar Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Adicionar target UEFI**:
   ```bash
   rustup target add x86_64-unknown-uefi
   ```

3. **Instalar ferramentas**:
   ```bash
   rustup component add rustfmt clippy
   ```

### Compilação

```bash
# Debug
cargo build --target x86_64-unknown-uefi

# Release
cargo build --target x86_64-unknown-uefi --release

# Com verificações
cargo clippy --target x86_64-unknown-uefi
cargo fmt --check
```

### Testes

```bash
# Testes unitários (quando disponíveis)
cargo test

# Teste em QEMU
# [Instruções específicas para testar em QEMU]
```

## Áreas que Precisam de Ajuda

Estamos especialmente interessados em contribuições nas seguintes áreas:

- [ ] **Secure Boot**: Implementação de validação de assinaturas
- [ ] **Modo de Recuperação**: Shell interativo de recuperação
- [ ] **Testes**: Testes unitários e de integração
- [ ] **Documentação**: Melhorias na documentação
- [ ] **Suporte a Arquiteturas**: Suporte para ARM64 (aarch64)
- [ ] **Performance**: Otimizações de carregamento

## Perguntas?

Se você tiver dúvidas sobre como contribuir:

1. Leia a [documentação](README.md)
2. Verifique as [issues existentes](https://github.com/redstone-os/ignite/issues)
3. Abra uma nova issue com sua pergunta

## Reconhecimento

Todos os contribuidores serão reconhecidos no arquivo [AUTHORS.md](AUTHORS.md).

---

Obrigado por contribuir para o Ignite! 🚀
