# Testes do Ignite Bootloader

[![Testes](https://img.shields.io/badge/testes-81_casos-brightgreen)]()
[![Cobertura](https://img.shields.io/badge/cobertura-em_desenvolvimento-yellow)]()

Suite completa de testes para o bootloader Ignite do Redstone OS.

## 📊 Resumo dos Testes

### Estatísticas

- **Total de Testes:** 81
- **Testes Unitários:** 66
  - ELF Parser: 12 testes
  - Config Parser: 20 testes
  - Boot Info: 17 testes
  - Memory Management: 17 testes
- **Testes de Integração:** 15
  - Boot Flow: 15 testes

## 🚀 Como Executar os Testes

### Executar Todos os Testes

```bash
cargo test --package ignite
```

### Executar Apenas Testes Unitários

```bash
cargo test --package ignite --lib
```

### Executar Apenas Testes de Integração

```bash
cargo test --package ignite --test '*'
```

### Executar Teste Específico

```bash
# Exemplo: testar apenas o ELF parser
cargo test --package ignite elf_parser

# Exemplo: testar apenas o config parser
cargo test --package ignite config_parser
```

### Executar com Output Detalhado

```bash
cargo test --package ignite -- --nocapture
```

### Executar em Modo Verbose

```bash
cargo test --package ignite -- --show-output
```

## 📁 Estrutura dos Testes

```
tests/
├── unit/                          # Testes unitários
│   ├── mod.rs                     # Módulo raiz
│   ├── elf_parser_tests.rs        # Testes do parser ELF
│   ├── config_parser_tests.rs     # Testes do parser de config
│   ├── boot_info_tests.rs         # Testes de boot info
│   └── memory_tests.rs            # Testes de memória
├── integration/                   # Testes de integração
│   ├── mod.rs                     # Módulo raiz
│   └── boot_flow_tests.rs         # Testes de fluxo completo
└── fixtures/                      # Arquivos de teste
    ├── README.md                  # Docs dos fixtures
    ├── sample_config.conf         # Config básica
    ├── sample_config_with_macros.conf
    └── sample_config_hierarchical.conf
```

## 📖 Documentação Completa

Para documentação detalhada sobre a estratégia de testes, casos cobertos e como adicionar novos testes, consulte:

- **[docs/TESTES.md](../docs/TESTES.md)** - Guia completo de testes (PT-BR)

## ✅ Casos de Teste Cobertos

### ELF Parser
- ✅ Parse de arquivos ELF válidos
- ✅ Validação de entry point
- ✅ Detecção de segmentos PT_LOAD
- ✅ Cálculo de ranges de endereços
- ✅ Rejeição de arquivos inválidos
- ✅ Validação de magic number
- ✅ Múltiplos segmentos carregáveis

### Config Parser
- ✅ Parse de configurações básicas
- ✅ Opções booleanas (yes/no)
- ✅ Timeout numérico e desabilitado
- ✅ Resoluções de vídeo
- ✅ Entradas de menu
- ✅ Macros e expansão
- ✅ Hierarquia de menus
- ✅ Comentários e linhas vazias

### Boot Info
- ✅ Criação de estruturas
- ✅ Validação de framebuffer
- ✅ Informações do kernel
- ✅ InitFS presente/ausente
- ✅ Memory regions
- ✅ Tipos de memória
- ✅ Serialização de dados

### Memory Management
- ✅ Bump allocator básico
- ✅ Alocação alinhada
- ✅ Múltiplas alocações
- ✅ Detecção de overflow
- ✅ Limites de memória
- ✅ Espaço usado/restante

### Boot Flow (Integração)
- ✅ Validação do fluxo completo
- ✅ Sequência de carregamento
- ✅ Validação de configuração
- ✅ Carregamento de kernel e InitFS
- ✅ Preparação de BootInfo
- ✅ Setup de framebuffer
- ✅ Memory map
- ✅ Handoff para kernel

## 🔧 Adicionando Novos Testes

1. **Identifique o módulo** apropriado (unit ou integration)
2. **Crie um teste** seguindo o padrão Arrange-Act-Assert
3. **Use nomes descritivos** que expliquem o que está sendo testado
4. **Adicione comentários** em português explicando o teste
5. **Execute** e verifique se passa
6. **Atualize esta documentação** se necessário

### Exemplo de Teste

```rust
#[test]
fn test_exemplo_descritivo() {
    // Arrange: Preparar dados de teste
    let dados = criar_dados_teste();
    
    // Act: Executar ação sendo testada
    let resultado = funcao_testada(dados);
    
    // Assert: Verificar resultado esperado
    assert_eq!(resultado, valor_esperado);
}
```

## 🐛 Debugging de Testes

Para debugar um teste específico:

```bash
# Executar um teste com output completo
cargo test --package ignite nome_do_teste -- --nocapture --show-output

# Executar com backtrace em caso de panic
RUST_BACKTRACE=1 cargo test --package ignite nome_do_teste
```

## 📝 Convenções

- **Nomes de testes:** Começam com `test_` seguido de descrição em snake_case
- **Comentários:** Sempre em português do Brasil
- **Arrange-Act-Assert:** Padrão usado em todos os testes
- **Fixtures:** Mantidos simples e documentados

## 🎯 Próximos Passos

- [ ] Implementar testes de filesystem (FAT32, ISO9660)
- [ ] Adicionar testes de GOP/vídeo
- [ ] Criar mocks para interfaces UEFI
- [ ] Configurar cobertura de código (tarpaulin)
- [ ] Adicionar testes de performance

---

**Última atualização:** 2025-12-21  
**Versão:** 0.4.0  
**Maintainer:** Equipe Redstone OS
