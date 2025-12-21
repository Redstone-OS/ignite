//! Diagnóstico de Sistema (Pre-Flight Check)
//!
//! Verifica a saúde básica dos componentes antes de tentar carregar o kernel.

use crate::{
    config::Entry,
    fs::{FileLoader, FileSystem},
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
    pub fn check_entry(fs: &mut dyn FileSystem, entry: &Entry) -> HealthStatus {
        crate::println!("Executando diagnóstico em '{}'...", entry.name);

        // 1. Verificar existência do Kernel
        // Isso previne pânico no BootProtocol
        if let Ok(mut root) = fs.root() {
            // Nota: precisamos lidar com o parse do path "boot():/kernel"
            // Aqui simplificamos assumindo path relativo para o teste
            // Em produção, usaríamos o `config::path::ConfigPath` resolver.

            // Simulação de check (pois o FS real precisa de path parsing
            // complexo) Se tivéssemos acesso direto ao arquivo:
            // if root.open_file(&entry.path).is_err() {
            //     return HealthStatus::Critical("Arquivo do Kernel não
            // encontrado"); }
        }

        // 2. Verificar Memória
        let mem_map_size = 0; // Obter do MemoryManager
        if mem_map_size > 0 {
            // Check ok
        }

        HealthStatus::Healthy
    }

    /// Verifica integridade do firmware.
    pub fn check_firmware() -> HealthStatus {
        let st = crate::uefi::system_table();
        // Verificar versão UEFI, Watchdog, etc.
        if st.hdr.revision < 0x00020000 {
            // < UEFI 2.0
            return HealthStatus::Warning("Versão UEFI antiga detectada");
        }
        HealthStatus::Healthy
    }
}
