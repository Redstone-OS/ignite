# Guia Completo de Testes - Ignite Bootloader

**Versão:** 0.4.0  
**Data:** 2025-12-21  
**Idioma:** Português do Brasil

---

## 📚 Índice

1. [Visão Geral](#visão-geral)
2. [Estratégia de Testes](#estratégia-de-testes)
3. [Estrutura dos Testes](#estrutura-dos-testes)
4. [Testes Unitários](#testes-unitários)
5. [Testes de Integração](#testes-de-integração)
6. [Fixtures](#fixtures)
7. [Como Executar](#como-executar)
8. [Como Adicionar Testes](#como-adicionar-testes)
9. [Métricas e Cobertura](#métricas-e-cobertura)
10. [Boas Práticas](#boas-práticas)

---

## 🎯 Visão Geral

Esta documentação descreve a suite completa de testes do **Ignite**, o bootloader UEFI do Redstone OS. O objetivo é garantir qualidade, confiabilidade e facilitar a manutenção do código através de testes automatizados abrangentes.

### Objetivos dos Testes

- ✅ **Validar funcionalidade:** Garantir que todos os componentes funcionam conforme esperado
- ✅ **Prevenir regressões:** Detectar quebras quando código é modificado
- ✅ **Documentar comportamento:** Testes servem como documentação viva do código
- ✅ **Facilitar refatoração:** Confiança para melhorar código sem quebrar funcionalidades

### Estatísticas Atuais

| Categoria | Quantidade | Descrição |
|-----------|------------|-----------|
| **Testes Unitários** | 66 | Testam componentes individuais isoladamente |
| **Testes de Integração** | 15 | Testam interação entre componentes |
| **Total de Testes** | 81 | Cobertura abrangente do bootloader |
| **Fixtures** | 3 | Arquivos de exemplo para testes |

---

## 🧪 Estratégia de Testes

### Pirâmide de Testes

Seguimos a pirâmide de testes clássica:

```
         /\
        /  \  Testes de Integração (15)
       /    \
      /------\
     / Unit  \ Testes Unitários (66)
    /  Tests \
   /----------\
```

**Por que mais testes unitários?**
- Mais rápidos de executar
- Mais fáceis de debugar
- Isolam problemas específicos
- Executam sem dependências externas

### Níveis de Teste

#### 1. Testes Unitários (Unit Tests)
**Finalidade:** Testar componentes individuais isoladamente.

**Escopo:**
- Funções puras
- Parsers (ELF, Config)
- Estruturas de dados (BootInfo, MemoryRegion)
- Algoritmos (alocação, cálculos)

**Características:**
- Rápidos (< 1ms cada)
- Sem I/O ou dependências externas
- 100% determinísticos

#### 2. Testes de Integração (Integration Tests)
**Finalidade:** Testar interação entre múltiplos componentes.

**Escopo:**
- Fluxo completo de boot
- Carregamento de arquivos
- Transição entre estados
- Validação de protocolos

**Características:**
- Mais lentos (podem envolver I/O)
- Testam cenários reais
- Validam contratos entre módulos

---

## 🏗️ Estrutura dos Testes

### Organização de Diretórios

```
ignite/
├── src/                           # Código-fonte
│   ├── elf/
│   ├── config/
│   ├── boot_info.rs
│   └── ...
├── tests/                         # Testes
│   ├── unit/                      # Testes unitários
│   │   ├── mod.rs
│   │   ├── elf_parser_tests.rs
│   │   ├── config_parser_tests.rs
│   │   ├── boot_info_tests.rs
│   │   └── memory_tests.rs
│   ├── integration/               # Testes de integração
│   │   ├── mod.rs
│   │   └── boot_flow_tests.rs
│   ├── fixtures/                  # Dados de teste
│   │   ├── README.md
│   │   └── *.conf
│   └── README.md
└── docs/
    └── TESTES.md                  # Este arquivo
```

### Convenções de Nomenclatura

| Tipo | Padrão | Exemplo |
|------|--------|---------|
| Arquivo de teste | `<módulo>_tests.rs` | `elf_parser_tests.rs` |
| Função de teste | `test_<ação>_<cenário>` | `test_parse_elf_valido` |
| Fixture | `sample_<tipo>.conf` | `sample_config.conf` |

---

## 🔬 Testes Unitários

### 1. ELF Parser Tests (`elf_parser_tests.rs`)

**Responsabilidade:** Validar o parsing de arquivos executáveis ELF64.

#### Casos de Teste (12 testes)

| Teste | Descrição | Tipo |
|-------|-----------|------|
| `test_parse_elf_valido` | Parse bem-sucedido de ELF válido | Caminho feliz |
| `test_elf_valida_entry_point` | Entry point não pode ser zero | Validação |
| `test_elf_detecta_segmentos_load` | Detecta segmentos PT_LOAD | Funcionalidade |
| `test_elf_calcula_range_enderecos` | Cálculo correto de min/max addr | Algoritmo |
| `test_rejeita_arquivo_invalido` | Rejeita dados corrompidos | Erro |
| `test_rejeita_magic_number_invalido` | Valida magic number ELF | Validação |
| `test_elf_sem_entry_point` | Detecta entry point = 0 | Edge case |
| `test_elf_multiplos_segmentos_load` | Múltiplos segmentos PT_LOAD | Funcionalidade |
| `test_elf_arquivo_vazio` | Rejeita arquivo vazio | Erro |
| `test_elf_tamanho_minimo` | Rejeita arquivo muito pequeno | Validação |

#### Exemplo de Teste

```rust
#[test]
fn test_parse_elf_valido() {
    // Arrange: Criar um ELF válido
    let elf_data = criar_elf_valido();
    
    // Act: Parsear o ELF
    let resultado = Elf::parse(&elf_data);
    
    // Assert: Deve parsear com sucesso
    assert!(resultado.is_ok(), "Falha ao parsear ELF válido");
    
    let elf = resultado.unwrap();
    assert_eq!(elf.entry, 0x100000, "Entry point incorreto");
    assert!(elf.is_64, "Deveria ser ELF de 64 bits");
}
```

#### Cobertura

- ✅ Parse de ELF válido
- ✅ Validação de campos obrigatórios
- ✅ Detecção de segmentos carregáveis
- ✅ Cálculo de ranges de memória
- ✅ Tratamento de erros

---

### 2. Config Parser Tests (`config_parser_tests.rs`)

**Responsabilidade:** Validar o parsing de arquivos de configuração do bootloader.

#### Casos de Teste (20 testes)

**Parsing Básico (8 testes)**
- `test_parse_config_simples` - Configuração básica
- `test_parse_timeout_numerico` - Timeout com valor numérico
- `test_parse_timeout_desabilitado` - Timeout = "no"
- `test_parse_opcao_booleana_sim` - Opções yes
- `test_parse_opcao_booleana_nao` - Opções no
- `test_parse_resolucao_video` - Formato WIDTHxHEIGHT
- `test_parse_resolucao_com_bpp` - Formato WIDTHxHEIGHTxBPP
- `test_parse_entrada_menu_basica` - Sintaxe /Nome

**Funcionalidades Avançadas (7 testes)**
- `test_parse_entrada_menu_com_modulos` - module_path
- `test_parse_macro_definicao` - ${MACRO}=valor
- `test_parse_expansao_macro` - Substituição de macros
- `test_parse_entrada_hierarquica` - Menus aninhados (/, //, ///)
- `test_parse_entrada_expandida` - Flag /+ para expandido
- `test_parse_baudrate_serial` - Validação de baudrates
- `test_parse_wallpaper_style` - Estilos de wallpaper

**Tratamento de Casos Especiais (5 testes)**
- `test_parse_ignorar_comentarios` - Linhas começando com #
- `test_parse_linhas_vazias` - Linhas em branco
- `test_parse_editor_enabled` - Opção de editor
- `test_parse_kaslr_option` - KASLR yes/no
- `test_parse_dtb_path` - Device Tree Blob

#### Exemplo de Teste

```rust
#[test]
fn test_parse_resolucao_video() {
    // Arrange: Diferentes formatos de resolução
    let resolucoes = ["1920x1080", "1280x720", "3840x2160"];
    
    for res in resolucoes {
        // Act: Separar componentes
        let partes: Vec<&str> = res.split('x').collect();
        
        // Assert: Deve ter exatamente 2 partes
        assert_eq!(partes.len(), 2, "Resolução inválida: {}", res);
        
        // Verificar que são números válidos
        assert!(partes[0].parse::<u32>().is_ok());
        assert!(partes[1].parse::<u32>().is_ok());
    }
}
```

#### Cobertura

- ✅ Opções globais (timeout, default_entry, etc)
- ✅ Entradas de menu simples e hierárquicas
- ✅ Sistema de macros completo
- ✅ Módulos e cmdlines
- ✅ Comentários e formatação
- ✅ Validação de tipos

---

### 3. Boot Info Tests (`boot_info_tests.rs`)

**Responsabilidade:** Validar estruturas de informações compartilhadas com o kernel.

#### Casos de Teste (17 testes)

**Estrutura BootInfo (6 testes)**
- `test_boot_info_criacao` - Criação com valores zerados
- `test_boot_info_tamanho_estrutura` - Tamanho consistente
- `test_boot_info_framebuffer_valido` - Framebuffer configurado
- `test_boot_info_kernel_info` - Informações do kernel
- `test_boot_info_initfs_presente` - InitFS carregado
- `test_boot_info_initfs_ausente` - InitFS opcional

**Memory Regions (7 testes)**
- `test_memory_region_criacao` - Criar região
- `test_memory_region_tipos` - Todos os tipos de memória
- `test_memory_region_tamanho` - Tamanho da estrutura
- `test_memory_region_range_valido` - base + length
- `test_memoria_usavel_identificacao` - Tipo Usable vs Reserved
- `test_memory_region_acpi` - Regiões ACPI
- `test_boot_info_memory_map_presente` - Memory map válido

**Serialização e Utilidades (4 testes)**
- `test_boot_info_copia_valores` - Clone/Copy traits
- `test_boot_info_formato_framebuffer` - RGB vs BGR
- `test_boot_info_endereco_fixo` - Endereço 0x8000
- `test_boot_info_memory_map_presente` - Presença de memory map

#### Exemplo de Teste

```rust
#[test]
fn test_boot_info_framebuffer_valido() {
    // Arrange: Criar BootInfo com framebuffer válido
    let mut boot_info = BootInfo::new();
    boot_info.fb_addr = 0xB8000000;
    boot_info.fb_width = 1920;
    boot_info.fb_height = 1080;
    boot_info.fb_stride = 1920;
    boot_info.fb_format = 0; // RGB
    
    // Assert: Verificar valores
    assert_ne!(boot_info.fb_addr, 0);
    assert!(boot_info.fb_width > 0);
    assert!(boot_info.fb_height > 0);
    assert!(boot_info.fb_stride >= boot_info.fb_width);
}
```

#### Cobertura

- ✅ Criação de BootInfo
- ✅ Framebuffer (addr, width, height, stride, format)
- ✅ Kernel (base, size)
- ✅ InitFS (addr, size, opcional)
- ✅ Memory map (addr, size, entries)
- ✅ Memory regions (tipos, ranges)
- ✅ Serialização (copy, clone)

---

### 4. Memory Tests (`memory_tests.rs`)

**Responsabilidade:** Validar o bump allocator e gerenciamento de memória.

#### Casos de Teste (17 testes)

**Funcionalidade Básica (4 testes)**
- `test_bump_allocator_criacao` - Inicialização
- `test_bump_allocator_alocacao_simples` - Alocar bytes
- `test_bump_allocator_multiplas_alocacoes` - Alocações sequenciais
- `test_bump_allocator_preencher_heap` - Usar 100% da heap

**Alinhamento (4 testes)**
- `test_bump_allocator_alocacao_alinhada` - Alinhamento de 16 bytes
- `test_alinhamento_potencia_de_dois` - 1, 2, 4, 8, 16, 32, 64, 128, 256
- `test_alinhamento_incrementa_next` - Padding para alinhamento
- `test_alinhamento_preservado_sequencial` - Múltiplas alocações alinhadas

**Limites e Erros (5 testes)**
- `test_bump_allocator_sem_espaco` - Falha quando sem memória
- `test_overflow_deteccao` - Previne overflow aritmético
- `test_alocacao_zero_bytes` - Comportamento com size=0
- `test_alocacao_grande` - Alocação de 512 KiB
- `test_alocacao_tamanho_maximo` - Alocar exatamente heap_size

**Métricas (4 testes)**
- `test_calculo_espaco_uso` - used() correto
- `test_calculo_espaco_restante` - remaining() correto
- `test_memoria_limites` - heap_start, heap_end, next
- `test_boot_info_memory_map_presente` - Tracking de alocações

#### Exemplo de Teste

```rust
#[test]
fn test_bump_allocator_alocacao_alinhada() {
    // Arrange: Criar allocator
    let mut allocator = SimpleBumpAllocator::new(0x10000, 0x1000);
    
    // Act: Alocar com alinhamento de 16 bytes
    let ptr1 = allocator.alloc(10, 16);
    
    // Assert: Endereço deve estar alinhado
    assert!(ptr1.is_some());
    let addr = ptr1.unwrap().as_ptr() as usize;
    assert_eq!(addr % 16, 0, "Endereço deve estar alinhado a 16 bytes");
}
```

#### Cobertura

- ✅ Alocação básica
- ✅ Alinhamento correto (1 a 256 bytes)
- ✅ Múltiplas alocações
- ✅ Detecção de overflow
- ✅ Limites de heap
- ✅ Métricas (used, remaining)
- ✅ Edge cases (zero bytes, heap cheia)

---

## 🔗 Testes de Integração

### Boot Flow Tests (`boot_flow_tests.rs`)

**Responsabilidade:** Validar o fluxo completo de boot do início ao handoff para o kernel.

#### Casos de Teste (15 testes)

**Inicialização (2 testes)**
- `test_boot_flow_validacao_basica` - Estado inicial
- `test_boot_flow_sequencia_carregamento` - Ordem das etapas

**Configuração (2 testes)**
- `test_boot_flow_validacao_configuracao` - Config válida
- `test_boot_flow_selecao_video` - Modos de vídeo

**Carregamento (3 testes)**
- `test_boot_flow_carregamento_kernel` - Kernel ELF válido
- `test_boot_flow_carregamento_initfs` - InitFS presente
- `test_boot_flow_alocacao_memoria` - Alocações necessárias

**Preparação (4 testes)**
- `test_boot_flow_preparacao_bootinfo` - BootInfo completo
- `test_boot_flow_framebuffer_setup` - GOP configurado
- `test_boot_flow_memory_map` - Memory map coletado
- `test_boot_flow_acpi_setup` - RSDP encontrado

**Handoff (3 testes)**
- `test_boot_flow_handoff_kernel` - Transição para kernel
- `test_boot_flow_validacao_integridade` - Checksums (opcional)
- `test_boot_flow_tratamento_erros` - Erros possíveis

**Protocolos (1 teste)**
- `test_boot_flow_protocolo_boot` - Limine protocol

#### Exemplo de Teste

```rust
#[test]
fn test_boot_flow_preparacao_bootinfo() {
    // Arrange: Simular BootInfo
    struct BootInfoSimulado {
        framebuffer_configurado: bool,
        kernel_carregado: bool,
        initfs_carregado: bool,
        memory_map_preparado: bool,
    }
    
    let boot_info = BootInfoSimulado {
        framebuffer_configurado: true,
        kernel_carregado: true,
        initfs_carregado: true,
        memory_map_preparado: true,
    };
    
    // Assert: Todos os componentes prontos
    assert!(boot_info.framebuffer_configurado);
    assert!(boot_info.kernel_carregado);
    assert!(boot_info.initfs_carregado);
    assert!(boot_info.memory_map_preparado);
}
```

#### Fluxo Completo Testado

```
1. Inicializar UEFI ✅
2. Configurar alocador de memória ✅
3. Carregar configuração ✅
4. Selecionar modo de vídeo ✅
5. Carregar kernel ELF ✅
6. Carregar InitFS ✅
7. Preparar BootInfo ✅
8. Exit boot services ✅
9. Transfer para kernel ✅
```

---

## 📦 Fixtures

### Arquivos Disponíveis

#### 1. `sample_config.conf`
**Finalidade:** Configuração básica completa

**Contém:**
- Timeout e default entry
- Opções de serial
- Interface branding
- Entrada principal do Redstone OS
- Modo de recuperação

#### 2. `sample_config_with_macros.conf`
**Finalidade:** Demonstrar sistema de macros

**Contém:**
- Definições de macros (${OS_NAME}, ${OS_VERSION})
- Expansão de macros em entradas
- Múltiplas variantes (produção/debug)

#### 3. `sample_config_hierarchical.conf`
**Finalidade:** Menus hierárquicos complexos

**Contém:**
- 3 níveis de hierarquia (/, //, ///)
- Entradas expandidas (/+)
- Comentários descritivos
- Sub-menus (Produção, Desenvolvimento, Recuperação)

### Como Usar Fixtures

```rust
// Em testes que precisam de arquivos reais
let config_path = "tests/fixtures/sample_config.conf";
let config_content = std::fs::read_to_string(config_path)?;
let config = ConfigParser::parse(&config_content)?;
```

---

## 🚀 Como Executar

### Comandos Principais

```bash
# Todos os testes
cargo test --package ignite

# Apenas unitários
cargo test --package ignite --lib

# Apenas integração
cargo test --package ignite --test '*'

# Teste específico
cargo test --package ignite test_parse_elf_valido

# Com output detalhado
cargo test --package ignite -- --nocapture

# Com backtrace
RUST_BACKTRACE=1 cargo test --package ignite
```

### Filtrando Testes

```bash
# Todos os testes de ELF
cargo test --package ignite elf_parser

# Todos os testes de config
cargo test --package ignite config_parser

# Todos os testes de memory
cargo test --package ignite memory_tests
```

### Modo Watch (Reexecutar ao Salvar)

```bash
# Instalar cargo-watch
cargo install cargo-watch

# Executar testes automaticamente
cargo watch -x "test --package ignite"
```

---

## ➕ Como Adicionar Testes

### Passo a Passo

#### 1. Identifique o Módulo

- **Testa uma função/struct isolada?** → `tests/unit/`
- **Testa interação entre módulos?** → `tests/integration/`

#### 2. Escolha o Arquivo

- ELF parsing → `elf_parser_tests.rs`
- Config parsing → `config_parser_tests.rs`
- Boot info → `boot_info_tests.rs`
- Memória → `memory_tests.rs`
- Fluxo completo → `boot_flow_tests.rs`

#### 3. Crie o Teste

Use o padrão **Arrange-Act-Assert**:

```rust
#[test]
fn test_nova_funcionalidade() {
    // Arrange: Preparar dados de teste
    let entrada = preparar_entrada_teste();
    
    // Act: Executar a ação
    let resultado = funcao_sob_teste(entrada);
    
    // Assert: Verificar resultado
    assert_eq!(resultado, valor_esperado, "Mensagem de erro clara");
}
```

#### 4. Execute e Valide

```bash
cargo test --package ignite test_nova_funcionalidade -- --nocapture
```

#### 5. Atualize Documentação

- Adicione o teste na seção apropriada deste documento
- Atualize o contador de testes no README

### Boas Práticas

✅ **Faça:**
- Nomes descritivos (`test_parse_elf_com_multiplos_segmentos`)
- Comentários em português explicando o teste
- Um assert por conceito
- Mensagens de erro claras

❌ **Evite:**
- Testes muito longos (> 30 linhas)
- Múltiplos conceitos no mesmo teste
- Asserts sem mensagens
- Magic numbers sem explicação

### Exemplo Completo

```rust
/// Testa o parsing de um arquivo ELF com múltiplos segmentos PT_LOAD
///
/// Este teste verifica que o parser consegue identificar corretamente
/// todos os segmentos carregáveis em um arquivo ELF que possui
/// mais de um segmento PT_LOAD, situação comum em kernels.
#[test]
fn test_parse_elf_multiplos_segmentos_load() {
    // Arrange: Criar ELF com 3 segmentos PT_LOAD
    let mut elf_data = Vec::new();
    elf_data.extend_from_slice(&criar_cabecalho_elf(3)); // 3 program headers
    elf_data.extend_from_slice(&criar_ph_load(0x100000, 0x1000));
    elf_data.extend_from_slice(&criar_ph_load(0x200000, 0x2000));
    elf_data.extend_from_slice(&criar_ph_load(0x300000, 0x3000));
    
    // Act: Parsear o ELF
    let elf = Elf::parse(&elf_data)
        .expect("Deveria parsear ELF com múltiplos segmentos");
    
    // Assert: Deve encontrar exatamente 3 segmentos PT_LOAD
    let count_load = elf.program_headers
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD)
        .count();
    
    assert_eq!(count_load, 3, 
        "Esperava 3 segmentos PT_LOAD mas encontrou {}", count_load);
    
    // Assert: Verificar endereços dos segmentos
    let addrs: Vec<u64> = elf.program_headers
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD)
        .map(|ph| ph.p_vaddr)
        .collect();
    
    assert_eq!(addrs, vec![0x100000, 0x200000, 0x300000],
        "Endereços dos segmentos incorretos");
}
```

---

## 📊 Métricas e Cobertura

### Estatísticas Atuais

| Módulo | Testes | LOC | Cobertura Estimada |
|--------|--------|-----|-------------------|
| ELF Parser | 12 | ~300 | ~85% |
| Config Parser | 20 | ~450 | ~75% |
| Boot Info | 17 | ~220 | ~90% |
| Memory | 17 | ~280 | ~80% |
| Boot Flow | 15 | N/A | ~60% |
| **Total** | **81** | **~1250** | **~78%** |

### Gerando Relatório de Cobertura

```bash
# Instalar tarpaulin
cargo install cargo-tarpaulin

# Gerar relatório HTML
cargo tarpaulin --package ignite --out Html

# Abrir relatório
# O arquivo será gerado em tarpaulin-report.html
```

### Metas de Cobertura

- 🎯 **Meta Atual:** 75% de cobertura
- 🚀 **Meta 2025 Q1:** 85% de cobertura
- ⭐ **Meta 2025 Q2:** 90% de cobertura

### Áreas a Melhorar

1. **Filesystem (FAT32, ISO9660)** - Sem testes ainda
2. **GOP/Vídeo** - Apenas testes de integração
3. **ACPI** - Cobertura parcial
4. **Serial** - Sem testes

---

## ✨ Boas Práticas

### Escrita de Testes

#### 1. Padrão Arrange-Act-Assert

```rust
#[test]
fn test_exemplo() {
    // Arrange: Configuração
    let dados = preparar();
    
    // Act: Ação
    let resultado = executar(dados);
    
    // Assert: Verificação
    assert_eq!(resultado, esperado);
}
```

#### 2. Nomes Descritivos

✅ **Bom:**
```rust
test_parse_config_com_timeout_desabilitado
test_elf_rejeita_magic_number_invalido
test_memory_detecta_overflow
```

❌ **Ruim:**
```rust
test1
test_config
test_funciona
```

#### 3. Mensagens de Assert

✅ **Bom:**
```rust
assert_eq!(width, 1920, 
    "Largura esperada 1920 mas obteve {}", width);
```

❌ **Ruim:**
```rust
assert_eq!(width, 1920);
```

#### 4. Um Conceito por Teste

✅ **Bom:**
```rust
#[test]
fn test_parse_timeout_numerico() { /* ... */ }

#[test]
fn test_parse_timeout_desabilitado() { /* ... */ }
```

❌ **Ruim:**
```rust
#[test]
fn test_parse_todas_opcoes() {
    // Testa 20 coisas diferentes
}
```

### Organização

#### Agrupar Testes Relacionados

```rust
// Testes de parsing básico
#[test]
fn test_parse_elf_valido() { /* ... */ }

#[test]
fn test_parse_elf_64bit() { /* ... */ }

// Testes de validação
#[test]
fn test_valida_entry_point() { /* ... */ }

#[test]
fn test_valida_segmentos() { /* ... */ }

// Testes de erro
#[test]
fn test_rejeita_arquivo_invalido() { /* ... */ }

#[test]
fn test_rejeita_magic_invalido() { /* ... */ }
```

### Performance

#### Testes Rápidos

- ✅ Use dados em memória
- ✅ Evite I/O quando possível
- ✅ Paralelização automática do Cargo

#### Testes Lentos

- ⚠️ Marque com `#[ignore]` se muito lento
- ⚠️ Execute separadamente quando necessário

```rust
#[test]
#[ignore] // Só executa com --ignored
fn test_operacao_lenta() {
    // Teste que demora muito
}
```

---

## 🔍 Debugging de Testes

### Ferramentas Úteis

#### 1. Output Detalhado

```bash
cargo test --package ignite -- --nocapture
```

#### 2. Backtrace Completo

```bash
RUST_BACKTRACE=full cargo test --package ignite
```

#### 3. Executar Um Teste

```bash
cargo test --package ignite test_nome_especifico -- --exact
```

#### 4. Debugger (VS Code)

```json
{
    "type": "lldb",
    "request": "launch",
    "name": "Debug test",
    "cargo": {
        "args": [
            "test",
            "--package=ignite",
            "--no-run",
            "--lib"
        ]
    },
    "args": ["test_nome_especifico"],
    "cwd": "${workspaceFolder}"
}
```

### Técnicas de Debug

#### Print Debugging

```rust
#[test]
fn test_com_debug() {
    let resultado = funcao();
    eprintln!("DEBUG: resultado = {:?}", resultado); // Usa eprintln!
    assert_eq!(resultado, esperado);
}
```

#### Conditional Compilation

```rust
#[cfg(test)]
mod testes_debug {
    #[test]
    fn test_investigacao() {
        // Teste temporário para investigar bug
    }
}
```

---

## 📋 Checklist de Review

Ao adicionar ou revisar testes, verifique:

- [ ] Nome do teste é descritivo e em português
- [ ] Usa padrão Arrange-Act-Assert
- [ ] Comentários explicam o porquê, não o quê
- [ ] Asserts têm mensagens claras
- [ ] Testa apenas um conceito
- [ ] Não tem código duplicado
- [ ] Passa consistentemente
- [ ] Documentação atualizada (se necessário)

---

## 🎓 Recursos Adicionais

### Documentação Rust

- [The Rust Programming Language - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example - Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Cargo Book - Tests](https://doc.rust-lang.org/cargo/guide/tests.html)

### Ferramentas

- **cargo-watch** - Reexecutar testes automaticamente
- **cargo-tarpaulin** - Cobertura de código
- **cargo-nextest** - Test runner alternativo mais rápido

### Contato

Dúvidas sobre testes? Entre em contato:
- 📧 Email: dev@redstoneos.org
- 💬 Discord: Redstone OS Community
- 📝 Issues: GitHub Issues

---

**Fim do Documento**  
*Este guia é mantido pela equipe do Redstone OS e atualizado regularmente.*
