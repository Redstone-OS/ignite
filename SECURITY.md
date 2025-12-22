# Security Policy

## 🔒 Política de Segurança - Ignite Bootloader

### Versões Suportadas

Atualmente, as seguintes versões do Ignite recebem atualizações de segurança:

| Versão | Suportada          | Status       |
|--------|-------------------|--------------|
| 0.1.x  | ✅ Sim            | Desenvolvimento Ativo |
| < 0.1  | ❌ Não            | Versões experimentais |

---

## 🐛 Reportando Vulnerabilidades

### Como Reportar

**NÃO abra issues públicas para vulnerabilidades de segurança.**

Se você descobrir uma vulnerabilidade de segurança no Ignite, por favor:

1. **Envie um email para**: security@redstone-os.org
2. **Assunto**: `[SECURITY] Ignite - Descrição breve`
3. **Inclua**:
   - Descrição detalhada da vulnerabilidade
   - Passos para reproduzir
   - Impacto potencial
   - Versão afetada
   - Proof of Concept (se disponível)
   - Sua informação de contato (opcional)

### Tempo de Resposta

- **Confirmação inicial**: Dentro de 48 horas
- **Avaliação completa**: Dentro de 7 dias
- **Patch de segurança**: Dentro de 30 dias (para vulnerabilidades críticas: 7 dias)

### Processo de Coordenação

1. **Recebimento**: Confirmaremos o recebimento do seu report
2. **Validação**: Verificaremos e validaremos a vulnerabilidade
3. **Desenvolvimento**: Criaremos um patch privado
4. **Teste**: Testaremos o patch completamente
5. **Divulgação**: Coordenaremos a divulgação pública
6. **Release**: Publicaremos atualização de segurança
7. **Advisory**: Publicaremos advisory de segurança

### Coordinated Disclosure

Seguimos a política de **Divulgação Coordenada**:

- ⏰ **90 dias** para divulgação pública após notificação
- 🔐 Manteremos confidencialidade até o patch estar disponível
- 📢 Publicaremos advisory no release
- 🏆 Reconheceremos o descobridor (se desejar)

---

## 🛡️ Recursos de Segurança

### Secure Boot

O Ignite suporta UEFI Secure Boot:

- ✅ Detecção automática de Secure Boot
- ✅ Validação de assinaturas
- ✅ Chainload apenas de binários assinados
- ✅ Políticas de segurança configuráveis

**Assinar o bootloader**:
```bash
sbsign --key db.key --cert db.crt ignite.efi --output ignite.efi.signed
```

### TPM (Trusted Platform Module)

Suporte a TPM 2.0 para Trusted Boot:

- ✅ Medição de binários (PCR 9)
- ✅ Extend de hashes
- ✅ Atestação remota
- ✅ Sealed secrets (futuro)

**PCRs utilizados**:
- **PCR 9**: Medições do bootloader
- **PCR 10**: Medições do kernel (futuro)

### Memory Safety

- 🦀 **Rust**: Memory-safe por design
- 🔒 **Minimal `unsafe`**: Apenas onde absolutamente necessário
- ✅ **Validação de entrada**: Todos os inputs são validados
- 🚫 **No buffer overflows**: Proteções automáticas do Rust
- 🔐 **NX bit**: Proteção contra execução de dados

### Input Validation

Todas as entradas são validadas:

- ✅ Paths (proteção contra path traversal)
- ✅ Configurações (validação de tipos)
- ✅ ELF binaries (magic bytes, estruturas)
- ✅ Filesystems (limites, offsets)

---

## 🔐 Security Best Practices

### Para Usuários

1. **Sempre use Secure Boot em produção**
   ```toml
   # ignite.conf
   [security]
   require_secure_boot = true
   ```

2. **Habilite medições TPM**
   ```toml
   [security]
   require_tpm = true
   tpm_pcr = 9
   ```

3. **Use senhas para configuração crítica** (futuro)

4. **Mantenha o firmware atualizado**

5. **Verifique checksums de releases**
   ```bash
   sha256sum -c checksums.txt
   ```

### Para Desenvolvedores

1. **Minimize código `unsafe`**
   - Documente TODOS os blocos `unsafe`
   - Justifique a necessidade
   - Valide todas as invariantes

2. **Valide TODAS as entradas**
   ```rust
   fn parse_input(data: &[u8]) -> Result<T> {
       if data.len() > MAX_SIZE {
           return Err(Error::TooLarge);
       }
       // ...
   }
   ```

3. **Use checked arithmetic**
   ```rust
   let result = value.checked_add(offset)?;
   ```

4. **Evite panic em bibliotecas**
   - Use `Result` em vez de `panic!`
   - Apenas `panic!` em `main.rs` para erros irrecuperáveis

5. **Audite dependências**
   ```bash
   cargo audit
   ```

---

## 🔍 Security Audits

### Auditorias Planejadas

- [ ] **Q1 2025**: Auditoria interna completa
- [ ] **Q2 2025**: Auditoria externa (TBD)
- [ ] **Q3 2025**: Fuzzing campaign

### Ferramentas Utilizadas

- ✅ **Cargo Clippy**: Lints de segurança
- ✅ **Cargo Audit**: Vulnerabilidades em dependências
- ⏳ **Cargo Fuzz**: Fuzzing (planejado)
- ⏳ **MIRI**: Undefined behavior detection (planejado)

---

## 📜 Vulnerabilidades Conhecidas

### CVE Database

Atualmente não há CVEs conhecidos para o Ignite.

Vulnerabilidades futuras serão listadas aqui com:
- CVE ID
- Descrição
- Severidade (CVSS)
- Versões afetadas
- Mitigação
- Patch disponível

---

## 🏆 Security Hall of Fame

Reconhecemos e agradecemos os pesquisadores que reportaram vulnerabilidades responsavelmente:

<!-- Lista será preenchida quando houver reports -->

_Nenhum report de segurança ainda. Seja o primeiro!_

---

## 📚 Recursos de Segurança

### Documentação

- [SEGURANCA.md](docs/SEGURANCA.md) - Documentação técnica completa
- [CONTRIBUINDO.md](docs/CONTRIBUINDO.md) - Políticas de código seguro

### Standards e Compliance

- ✅ **UEFI 2.10 Specification**
- ✅ **TCG PC Client Platform TPM Profile**
- ⏳ **Common Criteria** (futuro)

### External Resources

- [UEFI Security](https://uefi.org/specifications)
- [TPM 2.0 Library](https://trustedcomputinggroup.org/resource/tpm-library-specification/)
- [Rust Security](https://www.rust-lang.org/policies/security)

---

## 📞 Contato

- **Email de Segurança**: security@redstone-os.org
- **PGP Key**: [pending] (TBD)
- **GitHub Security Advisories**: Habilitado

---

## 📅 Atualizações

Esta política foi atualizada pela última vez em: **2025-12-21**

Revisamos esta política trimestralmente.

---

**Obrigado por ajudar a manter o Ignite seguro!** 🔒
