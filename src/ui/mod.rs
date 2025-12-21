//! Sistema de Interface de Usuário
//!
//! Menu de boot interativo, terminal e gráficos

pub mod editor;
pub mod input;
pub mod menu;
pub mod terminal;
pub mod theme;

pub use menu::BootMenu;
pub use terminal::GraphicalTerminal;
