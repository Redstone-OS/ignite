//! Módulo de gerenciamento de memória
//!
//! Responsável por alocação e gerenciamento de memória durante o processo de
//! boot

pub mod allocator;
pub mod bump_allocator;

pub use allocator::MemoryAllocator;
pub use bump_allocator::BumpAllocator;
