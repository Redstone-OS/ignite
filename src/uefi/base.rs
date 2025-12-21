//! Tipos Primitivos UEFI
//!
//! Define os blocos de construção básicos para comunicação com o firmware:
//! Handles (ponteiros opacos), GUIDs (identificadores únicos) e tipos de dados
//! simples.
//!
//! Referência: UEFI Spec 2.10, Seção 2.3

use core::fmt;

/// Handle UEFI - Um ponteiro opaco para um objeto gerido pelo firmware.
/// Usado para referenciar imagens, dispositivos, protocolos, etc.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Handle(pub *mut core::ffi::c_void);

impl Handle {
    /// Cria um Handle nulo.
    pub const fn null() -> Self {
        Handle(core::ptr::null_mut())
    }

    /// Verifica se o Handle é nulo ou inválido.
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle({:p})", self.0)
    }
}

/// GUID - Globally Unique Identifier (128-bit).
/// Usado extensivamente para identificar Protocolos e Tabelas de Configuração.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Guid {
    /// Construtor const para facilitar definições estáticas de Protocolos.
    pub const fn new(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> Self {
        Guid {
            data1: d1,
            data2: d2,
            data3: d3,
            data4: d4,
        }
    }
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

/// Evento assíncrono (Timer, Input, Signal).
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Event(pub *mut core::ffi::c_void);

/// Tipo Booleano UEFI (1 byte).
pub type Boolean = u8;
pub const TRUE: Boolean = 1;
pub const FALSE: Boolean = 0;

/// Caractere UCS-2 (UTF-16) usado em strings UEFI.
pub type Char16 = u16;
