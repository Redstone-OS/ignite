//! Interface de Usuário (UI)
//!
//! Gerencia a interação gráfica com o usuário, incluindo menu de boot,
//! renderização de fontes e tratamento de entrada.

pub mod font;
pub mod graphics;
pub mod input;
pub mod menu;
pub mod theme;

// Re-exports
pub use menu::Menu;
pub use theme::Theme;
