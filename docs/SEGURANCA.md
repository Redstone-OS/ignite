# Segurança - Ignite Bootloader

Documentação sobre recursos de segurança do Ignite.

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Secure Boot](#secure-boot)
- [TPM (Trusted Platform Module)](#tpm-trusted-platform-module)
- [Políticas de Segurança](#políticas-de-segurança)
- [Hardening](#hardening)
- [Chain of Trust](#chain-of-trust)

---

## Visão Geral

O Ignite implementa múltiplas camadas de segurança para garantir trusted boot e proteção contra malware.

### Princípios

1. **Verificação**: Validar cada componente antes da execução
2. **Medição**: Registrar hashes criptográficos no TPM
3. **Enforcement**: Políticas configuráveis de ação em caso de falha
4. **Minimização**: Superfície de ataque reduzida

---

## Secure Boot

### O que é Secure Boot

Secure Boot é um padrão UEFI que garante que apenas código assinado digitalmente seja executado durante o boot.

**Funcionamento**:
1. Firmware possui chaves públicas (PK, KEK, DB)
2. Bootloaders e kernels devem ser assinados
3. Assinatura é verificada antes da execução
4. Se verificação falhar, execução é bloqueada

---

### Detectar Estado do Secure Boot

O Ignite pode detectar se Secure Boot está ativo:

```rust
use ignite::security::secure_boot::{get_state, SecureBootState};

let state = get_state();
match state {
    SecureBootState::Enabled => {
        println!("Secure Boot está ativo");
    },
    SecureBootState::Disabled => {
        println!("Secure Boot desabilitado");
    },
    SecureBootState::SetupMode => {
        println!("Secure Boot em modo de configuração (sem chaves)");
    },
}
```

**Variáveis UEFI lidas**:
- `SecureBoot`: 1 se habilitado, 0 caso contrário
- `SetupMode`: 1 se em setup mode
- `BootMode`: Audit mode

---

### Assinar o Binário do Ignite

Para usar com Secure Boot, o `ignite.efi` deve ser assinado.

#### Gerar Chaves

```bash
# Gerar chave privada
openssl req -new -x509 -newkey rsa:2048 -keyout PK.key -out PK.crt -days 3650 -nodes -subj "/CN=Platform Key/"

# Converter para formato DER
openssl x509 -in PK.crt -outform DER -out PK.cer

# Criar arquivo ESL (EFI Signature List)
cert-to-efi-sig-list -g "$(uuidgen)" PK.crt PK.esl

# Assinar ESL com a própria chave (self-signed)
sign-efi-sig-list -k PK.key -c PK.crt PK PK.esl PK.auth
```

#### Assinar Bootloader

```bash
# Usando sbsign
sbsign --key DB.key --cert DB.crt --output ignite.efi.signed ignite.efi

# Renomear
mv ignite.efi.signed ignite.efi
```

#### Instalar Chaves no UEFI

**Método 1: Via UEFI Setup**
1. Entrar no UEFI Setup (F2/Del durante boot)
2. Navegar para "Secure Boot Configuration"
3. Escolher "Custom Mode"
4. Importar PK.auth, KEK.auth, DB.auth

**Método 2: Via efi-updatevar (Linux)**

```bash
sudo efi-updatevar -f PK.auth PK
sudo efi-updatevar -f KEK.auth KEK
sudo efi-updatevar -f DB.auth db
```

---

### Chainload com Secure Boot

Quando usando `protocol = "chainload"`, o próprio firmware UEFI valida a assinatura do binário carregado via `LoadImage()`. O Ignite não precisa fazer validação adicional.

---

### Validação Manual de Assinatura

Para kernels ELF não assinados via Authenticode (PE), o Ignite pode validar assinaturas GPG/PGP (futuro):

```ini
# ignite.conf (futuro)
[[entry]]
name = "Redstone OS"
protocol = "redstone"
path = "boot():/forge"
signature = "boot():/forge.sig"  # Assinatura GPG
public_key = "boot():/redstone-pubkey.asc"
```

---

## TPM (Trusted Platform Module)

### O que é TPM

TPM é um chip criptográfico que armazena chaves e pode realizar medições de integridade.

**PCRs (Platform Configuration Registers)**:
- Registradores que armazenam hashes
- Só podem ser estendidos (hash atual || novo hash)
- Não podem ser revertidos ou limpos (exceto com reboot)

---

### Medições do Ignite

O Ignite mede componentes críticos e estende PCRs:

| PCR | Conteúdo | Descrição |
|-----|----------|-----------|
| 0-7 | Firmware | Medido pelo UEFI firmware |
| 8 | Bootloader | Código do Ignite (medido pelo firmware) |
| 9 | Kernel | Kernel carregado (medido pelo Ignite) |
| 10-15 | Aplicações | Módulos/drivers (futuro) |

---

### Implementação

```rust
use ignite::security::tpm::measure_binary;

// Medir kernel no PCR 9
let kernel_data = fs::read_to_bytes(&mut kernel_file)?;
measure_binary(&kernel_data, 9, "kernel: forge")?;

// TPM internamente faz:
// PCR[9] = SHA256(PCR[9] || SHA256(kernel_data))
```

---

### Verificação Pós-Boot

Após boot, o sistema operacional pode ler PCRs e comparar com valores esperados (atestação):

```bash
# Ler PCRs no Linux
sudo tpm2_pcrread sha256

# Exemplo de output:
# sha256:
#   9: 0xABCDEF1234567890...  (hash do kernel)
```

Se o hash não corresponder ao esperado, significa que o kernel foi modificado.

---

### Atestação Remota

TPM permite atestação remota:

1. Servidor pede "quote" ao TPM
2. TPM assina PCRs com chave AIK (Attestation Identity Key)
3. Servidor verifica assinatura e PCRs
4. Se PCRs estiverem corretos, sistema é considerado confiável

---

## Políticas de Segurança

### Configurar Política

```rust
pub struct SecurityPolicy {
    pub require_secure_boot: bool,
    pub require_tpm_measurement: bool,
    pub on_validation_fail: PolicyAction,
}

pub enum PolicyAction {
    Halt,          // Parar boot (mais seguro)
    Warn,          // Avisar mas continuar
    RecoveryMode,  // Entrar em modo de recuperação
}
```

**Exemplo**:
```rust
let policy = SecurityPolicy {
    require_secure_boot: true,
    require_tpm_measurement: true,
    on_validation_fail: PolicyAction::Halt,
};

security::validate_and_measure(&kernel_data, "forge", &policy)?;
```

---

### Política em Config (futuro)

```ini
# ignite.conf
[security]
require_secure_boot = true
require_tpm = false
on_fail = "warn"  # halt | warn | recovery
```

---

## Hardening

### Técnicas Aplicadas

#### 1. **Rust Memory Safety**
- Sem buffer overflows
- Sem use-after-free
- Sem null pointer dereferences
- Sem data races (em código safe)

#### 2. **Minimal Unsafe**
- Unsafe apenas onde necessário (FFI, port I/O)
- Cada bloco unsafe documentado com `// SAFETY:`

#### 3. **NX Bit (No-Execute)**
```rust
const PAGE_NO_EXEC: u64 = 1 << 63;

// Páginas de dados marcadas como NX
page_table.map(data_addr, data_virt, PAGE_PRESENT | PAGE_WRITABLE | PAGE_NO_EXEC);
```

#### 4. **ASLR (Address Space Layout Randomization)**
Implementado via UEFI (firmware randomiza load address).

#### 5. **Stack Canaries**
Rust não emite stack canaries por padrão em `no_std`. Considerando implementação manual.

#### 6. **Input Validation**
Todos inputs (arquivos, configs) são validados antes de uso:

```rust
fn parse_config(data: &[u8]) -> Result<BootConfig> {
    // Validar tamanho máximo
    if data.len() > MAX_CONFIG_SIZE {
        return Err(BootError::Config(ConfigError::TooLarge));
    }
    
    // Validar UTF-8
    let text = core::str::from_utf8(data)
        .map_err(|_| BootError::Config(ConfigError::InvalidEncoding))?;
    
    // Parser com limites
    let config = parser::parse(text)?;
    
    // Validar valores
    if config.timeout > MAX_TIMEOUT {
        return Err(BootError::Config(ConfigError::InvalidTimeout));
    }
    
    Ok(config)
}
```

---

## Chain of Trust

### Cadeia de Confiança Completa

```
Hardware Root of Trust (TPM, Secure Boot hardware)
         ↓
UEFI Firmware (assinado pelo fabricante, medido em PCR 0-7)
         ↓
Ignite Bootloader (assinado com chave do projeto, medidoem PCR 8)
         ↓
Kernel (validado e medido pelo Ignite, medido em PCR 9)
         ↓
Init/Userspace (validado e medido pelo kernel, PCRs 10+)
```

Cada componente:
1. **Valida** o próximo (assinatura digital)
2. **Mede** o próximo (hash no TPM)
3. **Transfere controle** apenas se validação passar

---

### Quebra da Cadeia

Se qualquer etapa falhar:
- **Halt**: Boot para imediatamente
- **Warn**: Avisar usuário (visual + log)
- **Recovery**: Entrar em modo seguro

---

## Melhores Práticas

### Para Usuários

1. **Habilitar Secure Boot** no UEFI
2. **Habilitar TPM 2.0** no UEFI
3. **Assinar binários** customizados
4. **Monitorar PCRs** após updates
5. **Backup de chaves** Secure Boot

### Para Desenvolvedores

1. **Sempre validar inputs**
2. **Documentar unsafe**
3. **Usar Result em vez de panic**
4. **Adicionar testes de segurança**
5. **Auditar dependências**

---

## Vulnerabilidades Conhecidas

Nenhuma vulnerabilidade conhecida no momento.

**Reportar vulnerabilidades**:
- Email: security@redstone-os.org
- Ou: GitHub Security Advisories (privado)

---

**Última Atualização**: 2025-12-21
