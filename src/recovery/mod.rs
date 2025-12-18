//! Módulo de recuperação e fallback
//!
//! Responsável por sistema de fallback, modo de recuperação e diagnóstico

pub mod fallback;
pub mod keydetect;
pub mod diagnostics;

pub use fallback::{BootOptions, KernelEntry};
pub use keydetect::KeyDetector;
pub use diagnostics::Diagnostics;
