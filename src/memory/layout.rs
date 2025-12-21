//! Layout de Memória do Bootloader (Política de Endereçamento)
//!
//! Centraliza as constantes de onde o Kernel, Stack e Heap serão carregados.
//!
//! # Modelo de Memória Virtual
//! O Ignite segue o modelo "Higher Half Kernel":
//! - 0x0000_0000_0000_0000 -> Identity Map (até 4GiB ou mais, temporário)
//! - 0xFFFF_8000_0000_0000 -> Kernel Base (canônico)
//! - 0xFFFF_F000_0000_0000 -> Stack do Kernel e Heap

/// Endereço físico onde tentaremos carregar o Kernel (se possível).
/// 1MiB (marca histórica segura, evita BIOS area e IVT).
pub const KERNEL_LOAD_ADDR: u64 = 0x100_000;

/// Base Virtual onde o Kernel será linkado e executado.
/// Higer Half (-2GiB offset tipicamente em x86_64).
pub const KERNEL_VIRT_ADDR: u64 = 0xFFFF_8000_0000_0000;

/// Tamanho da Stack que o Bootloader prepara para o Kernel (64KiB).
/// Inclui uma "Guard Page" implícita se configurado na paginação.
pub const KERNEL_STACK_SIZE: u64 = 16 * 4096;

/// Tamanho da Heap do Bootloader (4MiB).
/// Suficiente para carregar cabeçalhos ELF e tabelas de paginação.
pub const BOOTLOADER_HEAP_SIZE: usize = 4 * 1024 * 1024;

/// Alinhamento padrão para páginas (4KiB).
pub const PAGE_SIZE: u64 = 4096;

/// Verifica se um endereço está alinhado com a página.
#[inline(always)]
pub fn is_aligned(addr: u64) -> bool {
    addr % PAGE_SIZE == 0
}
