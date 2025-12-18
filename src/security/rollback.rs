//! Proteção contra rollback de versão
//!
//! Impede que versões antigas e potencialmente vulneráveis sejam carregadas
//! NOTA: Opera em modo permissivo - alerta mas não bloqueia boot

use log::{info, warn};

/// Versão de kernel (semver simplificado)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KernelVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl KernelVersion {
    /// Cria uma nova versão
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Versão mínima permitida
    ///
    /// TODO: Carregar de configuração ou variável UEFI
    /// TODO: Permitir atualização dinâmica da versão mínima
    pub const MINIMUM_VERSION: KernelVersion = KernelVersion::new(0, 1, 0);

    /// Versão atual do bootloader
    pub const BOOTLOADER_VERSION: KernelVersion = KernelVersion::new(0, 2, 0);
}

/// Resultado de verificação de rollback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackResult {
    /// Versão permitida
    Allowed,
    /// Versão abaixo do mínimo permitido
    BelowMinimum,
}

/// Protetor contra rollback
pub struct RollbackProtection;

impl RollbackProtection {
    /// Verifica se versão é permitida
    ///
    /// Modo permissivo: alerta mas NÃO bloqueia boot
    ///
    /// # Argumentos
    /// * `version` - Versão do kernel a verificar
    ///
    /// # Retorna
    /// Resultado da verificação
    pub fn check_version(version: KernelVersion) -> RollbackResult {
        if version < KernelVersion::MINIMUM_VERSION {
            warn!("⚠ AVISO: Proteção contra rollback!");
            warn!("  Versão do kernel: {}.{}.{}", version.major, version.minor, version.patch);
            warn!("  Versão mínima:    {}.{}.{}", 
                KernelVersion::MINIMUM_VERSION.major,
                KernelVersion::MINIMUM_VERSION.minor,
                KernelVersion::MINIMUM_VERSION.patch
            );
            warn!("  Esta versão pode conter vulnerabilidades conhecidas.");
            warn!("  Continuando em modo permissivo...");
            RollbackResult::BelowMinimum
        } else {
            info!("✓ Versão {}.{}.{} permitida", version.major, version.minor, version.patch);
            RollbackResult::Allowed
        }
    }

    /// Extrai versão de kernel ELF
    ///
    /// TODO: Implementar extração real de seção .note ou header customizado
    /// Por enquanto, retorna versão padrão para não bloquear desenvolvimento
    ///
    /// # Argumentos
    /// * `elf_data` - Dados do arquivo ELF
    ///
    /// # Retorna
    /// Versão extraída ou None se não encontrada
    pub fn extract_version(_elf_data: &[u8]) -> Option<KernelVersion> {
        // TODO: Implementar extração real
        // Opções:
        // 1. Ler seção .note.kernel.version
        // 2. Ler header customizado
        // 3. Parsear string de versão em seção .rodata

        warn!("Extração de versão não implementada - usando versão padrão (TODO)");
        Some(KernelVersion::BOOTLOADER_VERSION)
    }

    /// Salva versão atual em variável UEFI
    ///
    /// TODO: Implementar persistência em variável UEFI
    /// Isso permite verificar rollback entre reboots
    pub fn save_current_version(_version: KernelVersion) {
        // TODO: Implementar
        // runtime_services.set_variable(
        //     "IgniteKernelVersion",
        //     &VENDOR_GUID,
        //     attributes,
        //     &version_bytes,
        // )
    }

    /// Carrega última versão conhecida de variável UEFI
    ///
    /// TODO: Implementar leitura de variável UEFI
    pub fn load_last_version() -> Option<KernelVersion> {
        // TODO: Implementar
        None
    }
}
