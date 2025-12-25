//! Gerenciamento de Secure Boot
//!
//! Detecta o estado de segurança do firmware lendo as variáveis globais EFI.
//! Referência: UEFI Spec 2.10, Seção 3.3 (Global Variables)


use crate::uefi::{
    base::{Guid, Status},
    system_table,
};

/// GUID para Variáveis Globais EFI (EfiGlobalVariable).
/// {8BE4DF61-93CA-11D2-AA0D-00E098032B8C}
pub const EFI_GLOBAL_VARIABLE: Guid = Guid::new(
    0x8BE4DF61,
    0x93CA,
    0x11d2,
    [0xAA, 0x0D, 0x00, 0xE0, 0x98, 0x03, 0x2B, 0x8C],
);

/// Estado atual do Secure Boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBootState {
    /// Secure Boot está ativo e aplicando políticas.
    Enforced,
    /// Secure Boot está desativado pelo usuário.
    Disabled,
    /// Sistema está em modo de configuração (Setup Mode).
    SetupMode,
    /// Estado indeterminado (erro ao ler variáveis).
    Unknown,
}

/// Verifica o estado do Secure Boot.
pub fn get_state() -> SecureBootState {
    let rt = system_table().runtime_services();

    // Nome das variáveis (UCS-2)
    // "SecureBoot\0"
    let sb_name: [u16; 11] = [
        'S' as u16, 'e' as u16, 'c' as u16, 'u' as u16, 'r' as u16, 'e' as u16, 'B' as u16,
        'o' as u16, 'o' as u16, 't' as u16, 0,
    ];

    // "SetupMode\0"
    let sm_name: [u16; 10] = [
        'S' as u16, 'e' as u16, 't' as u16, 'u' as u16, 'p' as u16, 'M' as u16, 'o' as u16,
        'd' as u16, 'e' as u16, 0,
    ];

    let mut data: u8 = 0;
    let mut data_size = 1;
    let mut attr = 0u32;

    // 1. Verificar SetupMode
    // Se SetupMode == 1, o Secure Boot não está operando normalmente (está
    // aprendendo chaves).
    let status_sm = unsafe {
        (rt.get_variable)(
            sm_name.as_ptr(),
            &EFI_GLOBAL_VARIABLE,
            &mut attr,
            &mut data_size,
            &mut data as *mut u8 as *mut core::ffi::c_void,
        )
    };

    if status_sm == Status::SUCCESS && data == 1 {
        return SecureBootState::SetupMode;
    }

    // 2. Verificar SecureBoot
    let status_sb = unsafe {
        (rt.get_variable)(
            sb_name.as_ptr(),
            &EFI_GLOBAL_VARIABLE,
            &mut attr,
            &mut data_size,
            &mut data as *mut u8 as *mut core::ffi::c_void,
        )
    };

    match status_sb {
        Status::SUCCESS => {
            if data == 1 {
                SecureBootState::Enforced
            } else {
                SecureBootState::Disabled
            }
        },
        _ => SecureBootState::Unknown,
    }
}

/// Verifica se devemos exigir assinaturas digitais.
pub fn enforcement_required() -> bool {
    matches!(get_state(), SecureBootState::Enforced)
}
