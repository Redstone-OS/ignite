# Fixtures de Teste - Ignite Bootloader

Este diretório contém arquivos de exemplo utilizados pelos testes do bootloader Ignite.

## 📁 Estrutura de Fixtures

### Configurações de Exemplo

- **`sample_config.conf`** - Arquivo de configuração básico do Ignite
- **`sample_config_with_macros.conf`** - Configuração com macros demonstrando expansão
- **`sample_config_hierarchical.conf`** - Configuração com menus hierárquicos

### Arquivos ELF

> **Nota:** Os arquivos ELF de teste são gerados programaticamente nos testes unitários
> devido ao tamanho e complexidade. Veja `tests/unit/elf_parser_tests.rs` para detalhes.

## 🎯 Uso nos Testes

Os fixtures são referenciados nos testes através de caminhos relativos:

```rust
let config_path = "tests/fixtures/sample_config.conf";
let config_content = std::fs::read_to_string(config_path).unwrap();
```

## ⚠️ Importante

- **Não modifique** os fixtures sem atualizar os testes correspondentes
- Mantenha os arquivos **pequenos e focados** para facilitar debugging
- **Documente** qualquer fixture novo adicionado aqui
- Use **comentários** nos arquivos de configuração para explicar recursos testados

## 📝 Adicionando Novos Fixtures

1. Crie o arquivo neste diretório
2. Documente sua finalidade neste README
3. Adicione testes que utilizem o fixture
4. Commit ambos simultaneamente

---

**Última atualização:** 2025-12-21  
**Maintainer:** Equipe Redstone OS
