//! Camada de Abstração UEFI (Unified Extensible Firmware Interface)
//!
//! Este módulo é o ponto central de acesso aos serviços do firmware.
//! Gerencia o Singleton da `SystemTable` e inicializa o ambiente.
//!
//! # Estrutura
//! - `base`: Tipos primitivos.
//! - `status`: Tratamento de erros.
//! - `table`: Acesso aos serviços (Boot, Runtime).
//! - `proto`: Drivers e Protocolos.

pub mod base;
pub mod proto;
pub mod status;
pub mod table;

// Re-exports para facilitar o uso em todo o projeto
pub use base::{Boolean, Char16, Event, Guid, Handle};
pub use status::{Result, Status};
pub use table::{boot::BootServices, runtime::RuntimeServices, system::SystemTable};

/// Referência global para a System Table.
/// Inicializada apenas uma vez no entry point `efi_main`.
static mut SYSTEM_TABLE: *mut SystemTable = core::ptr::null_mut();

/// Handle da Imagem do Bootloader.
static mut IMAGE_HANDLE: Handle = Handle(core::ptr::null_mut());

/// Inicializa o subsistema UEFI.
///
/// Deve ser chamado logo no início da função `efi_main`.
/// Configura o ponteiro global e desabilita o Watchdog Timer para evitar
/// resets.
///
/// # Safety
/// - Deve ser chamado apenas uma vez.
/// - Os ponteiros fornecidos pelo firmware devem ser válidos.
pub unsafe fn init(st: *mut SystemTable, image: Handle) {
    if !SYSTEM_TABLE.is_null() {
        panic!("BUG: UEFI System Table já inicializada!");
    }

    SYSTEM_TABLE = st;
    IMAGE_HANDLE = image;

    // Tenta desabilitar o Watchdog Timer padrão (geralmente 5 min).
    // Se falhar, continuamos, pois não é fatal imediatamente.
    if let Err(_e) = (*st).boot_services().set_watchdog_timer(0, 0) {
        // TODO: Logar aviso quando o sistema de log estiver ativo.
    }
}

/// Retorna uma referência segura e mutável para a System Table global.
///
/// # Panics
/// Panica se o UEFI não foi inicializado via `init()`.
pub fn system_table() -> &'static mut SystemTable {
    unsafe {
        if SYSTEM_TABLE.is_null() {
            panic!("BUG: Acesso à UEFI System Table antes da inicialização via uefi::init()");
        }
        &mut *SYSTEM_TABLE
    }
}

/// Retorna o Handle da imagem do Bootloader.
pub fn image_handle() -> Handle {
    unsafe { IMAGE_HANDLE }
}
