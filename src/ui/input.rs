//! Tratamento de Entrada (Teclado)
//!
//! Abstrai o protocolo `SimpleTextInput` do UEFI para eventos de alto nível.

use crate::uefi::{
    Status,
    base::Handle,
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
        let st = crate::uefi::system_table();
        // O cast é necessário pois con_in é *mut c_void na struct raw, mas sabemos que
        // é SimpleTextInput
        let protocol = st.con_in as *mut SimpleTextInputProtocol;
        Self { protocol }
    }

    /// Verifica se há uma tecla pressionada (não bloqueante).
    pub fn poll(&self) -> Option<Key> {
        let mut key = InputKey {
            scan_code:    0,
            unicode_char: 0,
        };

        unsafe {
            let status = ((*self.protocol).read_key_stroke)(self.protocol, &mut key);

            if status == Status::SUCCESS {
                Some(self.map_uefi_key(key))
            } else {
                None
            }
        }
    }

    /// Aguarda uma tecla (bloqueante).
    /// Usa `bs->wait_for_event` para economizar CPU.
    pub fn wait_for_key(&self) -> Key {
        let bs = crate::uefi::system_table().boot_services();

        loop {
            if let Some(k) = self.poll() {
                return k;
            }

            // Aguarda evento de teclado
            let event = unsafe { (*self.protocol).wait_for_key };
            let mut index = 0;
            unsafe {
                (bs.wait_for_event)(1, &mut event.clone(), &mut index);
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
                    13 => Key::Enter, // Carriage Return
                    8 => Key::Backspace,
                    c if c > 0 => Key::Char(char::from_u32(c as u32).unwrap_or('?')),
                    _ => Key::Unknown,
                }
            },
            _ => Key::Unknown,
        }
    }
}
