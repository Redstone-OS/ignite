# Ignite Builder - Menu Interativo

Sistema de build completo com interface rica para o bootloader Ignite.

## 📋 Requisitos

- Python 3.8+
- Rust toolchain (rustc, cargo, rustup)
- Target: `x86_64-unknown-uefi`

## 🚀 Instalação

```bash
# Instalar dependências Python
pip install -r requirements.txt

# Ou manualmente
pip install rich
```

## ▶️ Execução

```bash
python tools/ignite.py
```

## 🎯 Funcionalidades

### Build & Testes
- **[1] Build Debug** - Compilação rápida para desenvolvimento
- **[2] Build Release** - Compilação otimizada para produção
- **[3] Build Verbose** - Compilação com logs detalhados
- **[4] Todos os Testes** - Executa 81 casos de teste (unit + integration)
- **[5] Testes Unit** - Apenas testes unitários (66 testes)
- **[6] Testes Integration** - Apenas testes de integração (15 testes)

### Verificação & Distribuição
- **[7] Cargo Check** - Verificação básica de compilação
- **[8] Rustfmt Check** - Verifica formatação do código
- **[9] Clippy Lints** - Análise estática e sugestões
- **[10] Check Completo** - Todas as verificações
- **[11] Dist Release** - Cria distribuição otimizada em `dist/`
- **[12] Dist Debug** - Cria distribuição debug em `dist/`

### Utilidades
- **[13] Clean** - Limpa `target/` (artefatos de compilação)
- **[14] Clean All** - Limpa `target/` e `dist/`
- **[15] Doctor** - Diagnóstico completo do ambiente
- **[16] Ver Logs** - Lista logs recentes em `log/`
- **[Q] Sair** - Encerra o menu

## 📊 Recursos Visuais

### Progress Bars
Todas as operações longas (build, test, check) exibem barras de progresso em tempo real com:
- Spinner animado
- Barra de progresso
- Tempo decorrido
- Status colorido

### Tabelas Formatadas
Informações organizadas em tabelas coloridas:
- Doctor: ferramentas, projeto, estatísticas
- Logs: arquivos, tamanho, data
- Resultados: binários, tamanhos, timestamps

### Logs Automáticos
Tudo é registrado em `log/ignite_YYYYMMDD_HHMMSS.log`:
- Comandos executados
- Outputs e erros
- Timestamps
- Níveis de log (INFO, ERROR, DEBUG)

### Estatísticas
Rastreamento automático de:
- Builds realizados
- Testes executados
- Verificações
- Erros ocorridos
- Tempo de sessão

## 📁 Estrutura de Saída

### Distribuição (`dist/`)
```
dist/
├── EFI/
│   └── BOOT/
│       └── BOOTX64.EFI    # Bootloader UEFI
└── boot/
    └── ignite.conf        # Configuração
```

### Logs (`log/`)
```
log/
├── ignite_20251221_103045.log
├── ignite_20251221_113224.log
└── ...
```

## 🎨 Interface

O menu é organizado em **3 colunas**:
1. **Build & Testes** - Compilação e execução de testes
2. **Verificação & Dist** - Qualidade e distribuição
3. **Utilidades** - Ferramentas auxiliares

### Cores e Ícones
- 🟢 Verde: Sucesso
- 🔴 Vermelho: Erro
- 🟡 Amarelo: Avisos
- 🔵 Azul: Informação
- Emojis para cada tipo de operação

## 💡 Dicas de Uso

### Workflow de Desenvolvimento
```bash
1. Verificar ambiente
   → Opção 15 (Doctor)

2. Build debug
   → Opção 1

3. Executar testes
   → Opção 4

4. Verificar código
   → Opção 10
```

### Workflow de Release
```bash
1. Limpeza completa
   → Opção 14

2. Build release
   → Opção 2

3. Todos os testes
   → Opção 4

4. Criar distribuição
   → Opção 11
```

### Debugging
```bash
1. Build verbose
   → Opção 3

2. Ver logs
   → Opção 16

3. Doctor
   → Opção 15
```

## 🔧 Configuração

### Variáveis de Ambiente
Nenhuma configuração adicional necessária. O script detecta automaticamente:
- Diretório raiz do projeto
- Localização do Cargo
- Targets instalados

### Personalização
Edite `ignite.py` para personalizar:
- `LOG_DIR`: Diretório de logs (padrão: `log/`)
- `DIST_DIR`: Diretório de distribuição (padrão: `dist/`)
- Progress bar styles
- Cores e formatação

## 📝 Logs

### Formato
```
2025-12-21 11:03:45 [INFO] === BUILD DEBUG INICIADO ===
2025-12-21 11:03:45 [INFO] Executando: cargo build --package ignite --target x86_64-unknown-uefi
2025-12-21 11:04:12 [INFO] Build debug - Sucesso
2025-12-21 11:04:12 [INFO] Binário gerado: target/x86_64-unknown-uefi/debug/ignite.efi (2.34 MB)
2025-12-21 11:04:12 [INFO] === BUILD DEBUG FINALIZADO - SUCESSO ===
```

### Níveis
- **INFO**: Operações normais
- **ERROR**: Erros durante execução
- **DEBUG**: Outputs detalhados dos comandos

## 🐛 Troubleshooting

### "Biblioteca 'rich' não instalada"
```bash
pip install rich
```

### "Target não instalado"
```bash
rustup target add x86_64-unknown-uefi
```

### Menu quebrado/mal formatado
- Aumente o tamanho do terminal (mínimo 100x30)
- Use terminal com suporte a Unicode
- Windows: Use Windows Terminal ou PowerShell 7+

### Logs não aparecem
- Verifique permissões na pasta `log/`
- Execute com privilégios adequados

## 📈 Estatísticas de Exemplo

```
╔════════════════════════════════════════╗
║ 🚀 Ignite Builder                      ║
║ Sistema de Build Interativo            ║
║ Redstone OS | v0.4.0                   ║
║                                        ║
║ Sessão iniciada: 11:03:45              ║
║ Builds: 3 | Testes: 2 | Checks: 1      ║
╚════════════════════════════════════════╝
```

## 🎯 Próximos Passos

Após executar o menu:

1. **Doctor** (opção 15) - Verificar ambiente
2. **Build Debug** (opção 1) - Compilar
3. **Testes** (opção 4) - Validar
4. **Dist** (opção 11) - Criar distribuição

## 📚 Documentação Adicional

- [Testes](../tests/README.md) - Documentação dos testes
- [TESTES.md](../docs/TESTES.md) - Guia completo de testes
- [Ignite README](../README.md) - Documentação do bootloader

## 🤝 Contribuindo

Ao adicionar novas funcionalidades:

1. Adicione função específica
2. Registre logs apropriados
3. Adicione opção no menu
4. Atualize esta documentação
5. Teste com diferentes cenários

## 📄 Licença

Mesmo do projeto Redstone OS.

---

**Desenvolvido para Redstone OS**  
**Versão:** 1.0.0  
**Última atualização:** 2025-12-21
