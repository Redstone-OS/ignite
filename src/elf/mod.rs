//! Subsistema de Carregamento ELF
//!
//! Este módulo fornece a capacidade de parsear e carregar binários ELF64 na
//! memória, configurando corretamente as páginas físicas e o mapeamento
//! virtual.

pub mod header;
pub mod loader;

// O Parser agora é um detalhe interno do loader ou do header,
// não precisamos expô-lo diretamente a menos que seja para debug.
// Re-exportamos o Loader que é a interface principal.
pub use loader::ElfLoader;

// Re-exportar erros específicos se necessário
pub use crate::core::error::ElfError;
