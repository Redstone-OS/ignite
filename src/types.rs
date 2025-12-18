//! Tipos compartilhados do bootloader Ignite
//!
//! Este módulo define estruturas de dados compartilhadas entre diferentes módulos

/// Argumentos passados ao kernel durante o boot
#[repr(C, packed(8))]
#[derive(Debug, Clone, Copy)]
pub struct KernelArgs {
    /// Endereço base do kernel na memória
    pub kernel_base: u64,
    /// Tamanho do kernel em bytes
    pub kernel_size: u64,
    /// Endereço base da stack
    pub stack_base: u64,
    /// Tamanho da stack em bytes
    pub stack_size: u64,
    /// Endereço base das variáveis de ambiente
    pub env_base: u64,
    /// Tamanho das variáveis de ambiente
    pub env_size: u64,
    /// Endereço base da descrição de hardware (ACPI, etc)
    pub hwdesc_base: u64,
    /// Tamanho da descrição de hardware
    pub hwdesc_size: u64,
    /// Endereço base do mapa de áreas de memória
    pub areas_base: u64,
    /// Tamanho do mapa de áreas de memória
    pub areas_size: u64,
    /// Endereço base do InitFS (bootstrap)
    pub bootstrap_base: u64,
    /// Tamanho do InitFS
    pub bootstrap_size: u64,
}

impl KernelArgs {
    /// Cria uma nova estrutura KernelArgs com valores padrão
    pub fn new() -> Self {
        Self {
            kernel_base: 0x400000, // Base padrão do linker script
            kernel_size: 0,
            stack_base: 0,
            stack_size: 0,
            env_base: 0,
            env_size: 0,
            hwdesc_base: 0,
            hwdesc_size: 0,
            areas_base: 0,
            areas_size: 0,
            bootstrap_base: 0,
            bootstrap_size: 0,
        }
    }
}

impl Default for KernelArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// Informações sobre o framebuffer de vídeo
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    /// Ponteiro para o framebuffer
    pub ptr: u64,
    /// Tamanho do framebuffer em bytes
    pub size: usize,
    /// Resolução horizontal (largura)
    pub horizontal_resolution: usize,
    /// Resolução vertical (altura)
    pub vertical_resolution: usize,
    /// Stride (pixels por linha de scan)
    pub stride: usize,
}

/// Informações sobre um arquivo carregado
#[derive(Debug)]
pub struct LoadedFile {
    /// Ponteiro para os dados do arquivo na memória
    pub ptr: u64,
    /// Tamanho do arquivo em bytes
    pub size: usize,
}

/// Informações sobre o kernel ELF carregado
#[derive(Debug)]
pub struct LoadedKernel {
    /// Endereço base onde o kernel foi carregado
    pub base_address: u64,
    /// Tamanho total do kernel na memória
    pub size: u64,
    /// Ponto de entrada do kernel
    pub entry_point: u64,
}
