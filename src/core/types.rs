//! Tipos Básicos Compartilhados
//!
//! Estruturas simples de dados (POD - Plain Old Data) usadas para troca de
//! informações entre diferentes subsistemas (FS, Loader, Video) sem criar
//! dependências circulares.

/// Representa um arquivo carregado na memória.
/// Usado pelo FileSystem para retornar dados para o Loader.
#[derive(Debug, Clone, Copy)]
pub struct LoadedFile {
    /// Endereço físico do início do buffer.
    pub ptr:  u64,
    /// Tamanho do arquivo em bytes.
    pub size: usize,
}

/// Representa um Kernel carregado e pronto para execução.
#[derive(Debug, Clone, Copy)]
pub struct LoadedKernel {
    /// Endereço físico base onde o kernel foi carregado.
    pub base_address: u64,
    /// Ponto de entrada virtual (Entry Point) definido no cabeçalho ELF/PE.
    pub entry_point:  u64,
    /// Tamanho total ocupado na memória.
    pub size:         u64,
}

/// Informações básicas sobre o framebuffer (para uso interno antes do Handoff).
/// Nota: O `handoff::FramebufferInfo` é a estrutura final para o kernel
/// (repr(C)), esta aqui é para uso interno do Rust.
#[derive(Debug, Clone, Copy)]
pub struct Framebuffer {
    pub ptr:                   u64,
    pub size:                  usize,
    pub horizontal_resolution: usize,
    pub vertical_resolution:   usize,
    pub stride:                usize,
}
