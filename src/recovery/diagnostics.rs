//! Diagnóstico de Sistema (Pre-Flight Check)
//!
//! Verifica a saúde básica dos componentes antes de tentar carregar o kernel.
//! Garante que arquivos essenciais existam para evitar pânico no meio do boot.

use crate::{
    config::Entry,
    fs::{FileSystem, loader::FileLoader},
};

/// Resultado do diagnóstico.
#[derive(Debug, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning(&'static str),
    Critical(&'static str),
}

pub struct Diagnostics;

impl Diagnostics {
    /// Executa bateria de testes na entrada selecionada.
    ///
    /// Verifica se o kernel e os módulos (initrd) estão acessíveis.
    pub fn check_entry(fs: &mut dyn FileSystem, entry: &Entry) -> HealthStatus {
        crate::println!("Executando diagnóstico em '{}'...", entry.name);

        // Instancia um loader temporário para verificar arquivos
        let mut loader = FileLoader::new(fs);

        // 1. Verificar existência do Kernel
        if !loader.file_exists(&entry.path) {
            crate::println!("FALHA: Kernel '{}' não encontrado.", entry.path);
            return HealthStatus::Critical("Arquivo do Kernel ausente");
        }
        crate::println!("[92m[1m[OK][0m Kernel encontrado.");

        // 2. Verificar Módulos (Aviso)
        for module in &entry.modules {
            if !loader.file_exists(&module.path) {
                crate::println!("AVISO: Módulo '{}' não encontrado.", module.path);
                // Não retorna Critical pois o kernel pode bootar sem alguns módulos, mas avisa
                return HealthStatus::Warning("Módulo ausente");
            }
        }

        // 3. Verificar Memória (Opcional/Stub)
        // Em um sistema real, verificaríamos se há RAM suficiente para o tamanho do
        // kernel.

        HealthStatus::Healthy
    }

    /// Verifica integridade do firmware.
    pub fn check_firmware() -> HealthStatus {
        let st = crate::uefi::system_table();
        // Verificar revisão UEFI (Maior que 2.0 recomendado)
        // Revision: MSB 16 bits = Major, LSB 16 bits = Minor
        if st.hdr.revision < 0x00020000 {
            return HealthStatus::Warning(
                "Versão UEFI antiga detectada (< 2.0). Algumas features podem falhar.",
            );
        }
        HealthStatus::Healthy
    }
}
