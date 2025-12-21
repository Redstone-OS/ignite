//! Sistema de fallback para boot
//!
//! Gerencia tentativas de boot e seleção de kernel

/// Entrada de kernel no sistema de boot
#[derive(Debug, Clone)]
pub struct KernelEntry {
    /// Caminho do arquivo do kernel
    pub path:   &'static str,
    /// Nome amigável do kernel
    pub name:   &'static str,
    /// Caminho do InitFS (opcional)
    pub initfs: Option<&'static str>,
}

impl KernelEntry {
    /// Cria uma entrada de kernel
    pub fn new(path: &'static str, name: &'static str) -> Self {
        Self {
            path,
            name,
            initfs: Some("initfs"),
        }
    }

    /// Cria uma entrada de kernel sem InitFS
    pub fn new_without_initfs(path: &'static str, name: &'static str) -> Self {
        Self {
            path,
            name,
            initfs: None,
        }
    }
}

/// Opções de boot com fallback
pub struct BootOptions {
    /// Kernel principal (obrigatório)
    pub primary_kernel:  KernelEntry,
    /// Kernel de recuperação (opcional)
    /// TODO: Implementar kernel de recuperação dedicado
    pub recovery_kernel: Option<KernelEntry>,
    /// Contador de tentativas de boot
    /// TODO: Persistir este contador em variáveis UEFI
    pub boot_attempts:   u8,
    /// Máximo de tentativas antes de entrar em recovery
    pub max_attempts:    u8,
}

impl BootOptions {
    /// Cria opções de boot padrão
    pub fn default() -> Self {
        Self {
            primary_kernel:  KernelEntry::new("forge", "Redstone OS"),
            recovery_kernel: None, // TODO: Adicionar kernel de recovery
            boot_attempts:   0,
            max_attempts:    3, // Estilo Windows: 3 tentativas
        }
    }

    /// Seleciona qual kernel usar baseado no contador de tentativas
    pub fn select_kernel(&self) -> &KernelEntry {
        if self.boot_attempts >= self.max_attempts {
            // Tentar recovery se disponível
            if let Some(ref recovery) = self.recovery_kernel {
                log::warn!(
                    "Múltiplas falhas detectadas ({}/{}). Usando kernel de recuperação.",
                    self.boot_attempts,
                    self.max_attempts
                );
                return recovery;
            }
        }

        // Usar kernel principal
        &self.primary_kernel
    }

    /// Incrementa contador de tentativas
    /// TODO: Persistir em variável UEFI para sobreviver a reboots
    pub fn increment_attempts(&mut self) {
        self.boot_attempts += 1;
        log::info!(
            "Tentativa de boot: {}/{}",
            self.boot_attempts,
            self.max_attempts
        );
    }

    /// Reseta contador após boot bem-sucedido
    /// TODO: Limpar variável UEFI
    pub fn reset_attempts(&mut self) {
        if self.boot_attempts > 0 {
            log::info!("Boot bem-sucedido. Resetando contador de tentativas.");
            self.boot_attempts = 0;
        }
    }

    /// Verifica se deve entrar em modo de recuperação
    pub fn should_enter_recovery(&self) -> bool {
        self.boot_attempts >= self.max_attempts && self.recovery_kernel.is_some()
    }
}

/// Resultado de tentativa de boot
pub enum BootAttemptResult {
    /// Boot bem-sucedido (nunca retorna na prática)
    Success,
    /// Boot falhou, pode tentar novamente
    Failed,
    /// Deve entrar em modo de recuperação
    EnterRecovery,
}
