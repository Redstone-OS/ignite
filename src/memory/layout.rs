//! Layout de Memória do Bootloader (Política de Endereçamento)
//!
//! Centraliza as constantes de onde o Kernel, Stack e Heap serão carregados.

/// Endereço físico onde tentaremos carregar o Kernel (se possível).
pub const KERNEL_LOAD_ADDR: u64 = 0x100_000;

/// Base Virtual onde o Kernel será linkado e executado.
/// Higher Half (-2GiB offset tipicamente em x86_64).
pub const KERNEL_VIRT_ADDR: u64 = 0xFFFF_8000_0000_0000;

/// Tamanho da Stack que o Bootloader prepara para o Kernel (64KiB).
pub const KERNEL_STACK_SIZE: u64 = 64 * 1024;

/// Tamanho da Heap do Bootloader (4MiB).
pub const BOOTLOADER_HEAP_SIZE: usize = 4 * 1024 * 1024;

/// Alinhamento padrão para páginas (4KiB).
pub const PAGE_SIZE: u64 = 4096;

/// Verifica se um endereço está alinhado com a página.
#[inline(always)]
pub fn is_aligned(addr: u64) -> bool {
    addr % PAGE_SIZE == 0
}

/// Layout de memória configurado pelo bootloader.
/// Usado para informar ao Kernel onde os segmentos foram carregados.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootLayout {
    /// Início físico do Kernel.
    pub kernel_phys:  u64,
    /// Início virtual do Kernel.
    pub kernel_virt:  u64,
    /// Tamanho do Kernel.
    pub kernel_size:  u64,
    /// Topo da pilha (endereço mais alto).
    pub stack_top:    u64,
    /// Base da pilha (endereço mais baixo).
    pub stack_bottom: u64,
}
