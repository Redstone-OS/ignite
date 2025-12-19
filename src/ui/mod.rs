//! User Interface System
//!
//! Interactive boot menu, terminal, and graphics

pub mod editor;
pub mod input;
pub mod menu;
pub mod terminal;
pub mod theme;

pub use menu::BootMenu;
pub use terminal::GraphicalTerminal;
