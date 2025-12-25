//! Tratamento de Entrada (Teclado)
//!
//! Abstrai o protocolo `SimpleTextInput` do UEFI para eventos de alto nível.
//! Permite navegação nos menus e detecção de teclas de recuperação.

use crate::uefi::{
    Status,
    system_table,
    table::system::{InputKey, SimpleTextInputProtocol},
};

/// Teclas especiais mapeadas do UEFI Scan Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Escape,
    Backspace,
    Char(char),
    Unknown,
}

pub struct InputManager {
    protocol: *mut SimpleTextInputProtocol,
}

impl InputManager {
    /// Inicializa o gerenciador de input usando o STDIN do sistema.
    pub fn new() -> Self {
        let st = system_table();
        // O cast é seguro aqui pois sabemos que con_in segue a ABI do SimpleTextInput
        let protocol = st.con_in;
        Self { protocol }
    }

    /// Verifica se há uma tecla pressionada (não bloqueante).
    pub fn poll(&self) -> Option<Key> {
        let mut key = InputKey::default();

        unsafe {
            // Chama ReadKeyStroke (função FFI)
            let status = ((*self.protocol).read_key_stroke)(self.protocol, &mut key);

            if status == Status::SUCCESS {
                Some(self.map_uefi_key(key))
            } else {
                None
            }
        }
    }

    /// Aguarda uma tecla (bloqueante).
    /// Usa `bs->wait_for_event` para economizar CPU em vez de spinloop.
    pub fn wait_for_key(&self) -> Key {
        let bs = system_table().boot_services();

        loop {
            if let Some(k) = self.poll() {
                return k;
            }

            // Aguarda evento de teclado (interrupção/sinal)
            unsafe {
                let event = (*self.protocol).wait_for_key;
                let mut index = 0;
                // Wait for event (bloqueia CPU até interrupção)
                // OBS: wait_for_event_f foi adicionado na refatoração do boot.rs
                (bs.wait_for_event_f)(1, &mut event.clone(), &mut index);
            }
        }
    }

    fn map_uefi_key(&self, key: InputKey) -> Key {
        // Scan codes UEFI (Spec 12.3)
        match key.scan_code {
            0x01 => Key::Up,
            0x02 => Key::Down,
            0x03 => Key::Right,
            0x04 => Key::Left,
            0x17 => Key::Escape,
            0 => {
                // Se scan_code é 0, usamos unicode_char
                match key.unicode_char {
                    13 => Key::Enter,    // Carriage Return
                    8 => Key::Backspace, // Backspace
                    c if c > 0 => Key::Char(char::from_u32(c as u32).unwrap_or('?')),
                    _ => Key::Unknown,
                }
            },
            _ => Key::Unknown,
        }
    }
}
