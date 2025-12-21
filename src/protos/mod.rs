//! Abstração de Protocolos de Boot
//!
//! Gerencia o carregamento de diferentes formatos de kernel (Nativo, Linux,
//! Multiboot2). O objetivo é preparar o estado da máquina para o salto final.

use alloc::vec::Vec;

use crate::core::{error::Result, types::LoadedFile};

pub mod chainload;
pub mod linux;
pub mod multiboot2;
pub mod redstone;

/// Informações necessárias para executar o kernel (Registradores e Ponteiros).
#[derive(Debug, Clone, Copy, Default)]
pub struct KernelLaunchInfo {
    /// Endereço virtual de entrada (RIP).
    pub entry_point:   u64,
    /// Ponteiro da Stack inicial (RSP), se o protocolo exigir que o bootloader
    /// a configure.
    pub stack_pointer: Option<u64>,
    /// Valor para o registrador RDI (1º Argumento - System V AMD64).
    /// Usado pelo Redstone (ponteiro para BootInfo).
    pub rdi:           u64,
    /// Valor para o registrador RSI (2º Argumento).
    /// Usado pelo Linux (ponteiro para boot_params).
    pub rsi:           u64,
    /// Valor para o registrador RDX (3º Argumento).
    pub rdx:           u64,
    /// Valor para o registrador RBX.
    /// Usado pelo Multiboot2 (ponteiro para MBI).
    pub rbx:           u64,
}

/// Interface que todo carregador de kernel deve implementar.
pub trait BootProtocol {
    /// Nome do protocolo (para logs).
    fn name(&self) -> &str;

    /// Verifica se este protocolo suporta o arquivo fornecido (Magic Bytes).
    fn identify(&self, file_content: &[u8]) -> bool;

    /// Carrega o kernel na memória e prepara as estruturas de dados.
    ///
    /// # Argumentos
    /// * `kernel_file`: Conteúdo do kernel.
    /// * `cmdline`: String de argumentos do kernel.
    /// * `modules`: Lista de arquivos auxiliares (InitRD, Drivers) já
    ///   carregados.
    fn load(
        &mut self,
        kernel_file: &[u8],
        cmdline: Option<&str>,
        modules: Vec<LoadedFile>,
    ) -> Result<KernelLaunchInfo>;
}

/// Tenta detectar e carregar um kernel usando todos os protocolos disponíveis.
pub fn load_any(
    allocator: &mut crate::memory::FrameAllocator, // Abstração necessária
    page_table: &mut crate::memory::PageTableManager,
    kernel_file: &[u8],
    cmdline: Option<&str>,
    modules: Vec<LoadedFile>,
) -> Result<KernelLaunchInfo> {
    // Lista de protocolos suportados
    // Nota: Em um sistema real, você instanciaria isso de forma mais dinâmica
    // ou passaria as dependências (alocador) via construtor.

    // 1. Tentar Protocolo Nativo (Redstone/ELF)
    let mut redstone = redstone::RedstoneProtocol::new(allocator, page_table);
    if redstone.identify(kernel_file) {
        crate::println!("Detectado Kernel Redstone/ELF.");
        return redstone.load(kernel_file, cmdline, modules);
    }

    // 2. Tentar Linux
    let mut linux = linux::LinuxProtocol::new(allocator);
    if linux.identify(kernel_file) {
        crate::println!("Detectado Kernel Linux (bzImage).");
        return linux.load(kernel_file, cmdline, modules);
    }

    // 3. Tentar Multiboot2
    let mut mb2 = multiboot2::Multiboot2Protocol::new(allocator);
    if mb2.identify(kernel_file) {
        crate::println!("Detectado Kernel Multiboot2.");
        return mb2.load(kernel_file, cmdline, modules);
    }

    Err(crate::core::error::BootError::Generic(
        "Formato de kernel desconhecido",
    ))
}
