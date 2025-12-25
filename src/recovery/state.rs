//! Estado de Boot Persistente
//!
//! Gerencia as variáveis de ambiente UEFI para rastrear falhas de boot
//! e tentativas de recuperação entre reinicializações.

use core::mem::size_of;

use crate::uefi::{
    base::{Guid, Status},
    system_table,
};

/// GUID da variável de estado do Ignite (Vendor GUID).
/// {4a67b082-0a4c-41cf-b6c7-440b29bb8c4f}
pub const IGNITE_VENDOR_GUID: Guid = Guid::new(
    0x4a67b082,
    0x0a4c,
    0x41cf,
    [0xb6, 0xc7, 0x44, 0x0b, 0x29, 0xbb, 0x8c, 0x4f],
);

/// Nome da variável de estado.
const STATE_VAR_NAME: [u16; 12] = [
    'I' as u16, 'g' as u16, 'n' as u16, 'B' as u16, 'o' as u16, 'o' as u16, 't' as u16, 'S' as u16,
    't' as u16, 'a' as u16, 't' as u16, 0,
];

/// Atributos da variável (Non-Volatile + BootService + Runtime).
const VAR_ATTR: u32 = 0x00000007;

/// Estrutura persistida na NVRAM.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PersistentState {
    /// Número de tentativas falhas consecutivas.
    pub failed_attempts: u8,
    /// Último índice de entrada tentado.
    pub last_entry_idx:  u8,
    /// Flags de estado (ex: Forçar Recovery no próximo boot).
    pub flags:           u8,
    /// Checksum simples para validar a estrutura.
    pub checksum:        u8,
}

impl PersistentState {
    /// Tenta carregar o estado da NVRAM.
    pub fn load() -> Self {
        let rt = system_table().runtime_services();

        // Como não temos um wrapper "safe" completo para GetVariable com alocação no
        // core uefi ainda, usamos um buffer fixo na stack.
        let mut data = [0u8; size_of::<PersistentState>()];
        let mut size = data.len();
        let mut attr = 0u32;

        let status = unsafe {
            (rt.get_variable)(
                STATE_VAR_NAME.as_ptr(),
                &IGNITE_VENDOR_GUID,
                &mut attr,
                &mut size,
                data.as_mut_ptr() as *mut core::ffi::c_void,
            )
        };

        if status == Status::SUCCESS && size == size_of::<PersistentState>() {
            let state: PersistentState = unsafe { core::ptr::read(data.as_ptr() as *const _) };
            // TODO: Validar checksum
            state
        } else {
            // Se não existir ou erro, retorna estado limpo
            Self::default()
        }
    }

    /// Salva o estado atual na NVRAM.
    pub fn save(&self) {
        let rt = system_table().runtime_services();
        let size = size_of::<PersistentState>();

        unsafe {
            (rt.set_variable)(
                STATE_VAR_NAME.as_ptr(),
                &IGNITE_VENDOR_GUID,
                VAR_ATTR,
                size,
                self as *const _ as *mut core::ffi::c_void,
            );
        }
    }

    /// Registra uma nova tentativa de boot.
    pub fn mark_attempt(&mut self, entry_idx: usize) {
        self.failed_attempts = self.failed_attempts.saturating_add(1);
        self.last_entry_idx = entry_idx as u8;
        self.save();
    }

    /// Reseta o contador de falhas (chamar após boot com sucesso, via OS agent
    /// ou script). Nota: O bootloader só pode fazer isso se tiver certeza
    /// do sucesso, o que é difícil. Geralmente o OS que limpa essa flag
    /// após subir o Init System.
    pub fn reset(&mut self) {
        self.failed_attempts = 0;
        self.flags = 0;
        self.save();
    }
}
