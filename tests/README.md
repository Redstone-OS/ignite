# Guia de Testes - Ignite Bootloader

## 📋 Estratégia de Testes

O Ignite possui uma suíte de testes completa de nível industrial para garantir estabilidade e confiabilidade.

## Tipos de Testes

### 1. Testes Unitários
- **Localização**: `tests/unit/`
- **Propósito**: Testar funções e módulos isoladamente
- **Execução**: `cargo test --lib`

### 2. Testes de Integração
- **Localização**: `tests/integration/`
- **Propósito**: Testar interação entre módulos
- **Execução**: `cargo test --test integration_tests`

### 3. Testes de Propriedade
- **Localização**: `tests/property/`
- **Propósito**: Testar propriedades invariantes
- **Execução**: `cargo test --test property_tests`

### 4. Testes de Fuzzing
- **Localização**: `fuzz/`
- **Propósito**: Encontrar bugs com inputs aleatórios
- **Execução**: `cargo fuzz run <target>`

## Cobertura de Testes

### Módulos Testados

| Módulo | Unitários | Integração | Propriedade | Cobertura |
|--------|-----------|------------|-------------|-----------|
| config | ✅ | ✅ | ✅ | ~95% |
| memory | ✅ | ✅ | ✅ | ~90% |
| elf | ✅ | ✅ | ✅ | ~95% |
| fs | ✅ | ✅ | ❌ | ~80% |
| security | ✅ | ❌ | ❌ | ~70% |

## Executar Todos os Testes

```bash
# Todos os testes
cargo test

# Com output verboso
cargo test -- --nocapture

# Testes específicos
cargo test config::
cargo test memory::
```

## CI/CD

Testes são executados automaticamente no GitHub Actions em cada commit e PR.

---

**Última Atualização**: 2025-12-21
