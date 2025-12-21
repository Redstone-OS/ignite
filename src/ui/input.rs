//! Manipulação de Entrada
//!
//! Processamento de entrada de teclado e mouse

/// Códigos de tecla
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Char(char),
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Manipulador de entrada
pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }

    /// Aguardar pressionamento de tecla (bloqueante)
    pub fn wait_key(&self) -> Option<Key> {
        // TODO: Implementar leitura real de tecla da UEFI
        // Por enquanto, retorna None
        None
    }

    /// Verificar se há tecla disponível (não bloqueante)
    pub fn key_available(&self) -> bool {
        // TODO: Implementar
        false
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
