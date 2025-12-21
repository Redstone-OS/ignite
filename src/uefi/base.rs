//! Tipos Básicos UEFI
//!
//! Referência: UEFI Spec 2.10, Seção 2.3 - Data Types

use core::{fmt, ptr::NonNull};

/// Handle UEFI - ponteiro opaco para objetos UEFI
///
/// Spec: 2.3.1 - Data Types
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Handle(pub *mut core::ffi::c_void);

impl Handle {
    /// Cria um Handle nulo
    pub const fn null() -> Self {
        Handle(core::ptr::null_mut())
    }

    /// Verifica se o Handle é nulo
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Cria Handle a partir de ponteiro
    pub fn from_ptr(ptr: *mut core::ffi::c_void) -> Option<Self> {
        NonNull::new(ptr).map(|p| Handle(p.as_ptr()))
    }

    /// Obtém o ponteiro interno
    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.0
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Handle({:p})", self.0)
    }
}

/// Status Code UEFI
///
/// Spec: Appendix D - Status Codes
///
/// Status codes são valores usize onde:
/// - Bit mais significativo = 0: Sucesso ou Warning
/// - Bit mais significativo = 1: Erro
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Status(pub usize);

impl Status {
    // === SUCCESS ===
    pub const SUCCESS: Status = Status(0);

    // === WARNINGS (high bit clear) ===
    pub const WARN_UNKNOWN_GLYPH: Status = Status(1);
    pub const WARN_DELETE_FAILURE: Status = Status(2);
    pub const WARN_WRITE_FAILURE: Status = Status(3);
    pub const WARN_BUFFER_TOO_SMALL: Status = Status(4);
    pub const WARN_STALE_DATA: Status = Status(5);
    pub const WARN_FILE_SYSTEM: Status = Status(6);
    pub const WARN_RESET_REQUIRED: Status = Status(7);

    // === ERRORS (high bit set) ===
    pub const LOAD_ERROR: Status = Status(1 | (1 << (usize::BITS - 1)));
    pub const INVALID_PARAMETER: Status = Status(2 | (1 << (usize::BITS - 1)));
    pub const UNSUPPORTED: Status = Status(3 | (1 << (usize::BITS - 1)));
    pub const BAD_BUFFER_SIZE: Status = Status(4 | (1 << (usize::BITS - 1)));
    pub const BUFFER_TOO_SMALL: Status = Status(5 | (1 << (usize::BITS - 1)));
    pub const NOT_READY: Status = Status(6 | (1 << (usize::BITS - 1)));
    pub const DEVICE_ERROR: Status = Status(7 | (1 << (usize::BITS - 1)));
    pub const WRITE_PROTECTED: Status = Status(8 | (1 << (usize::BITS - 1)));
    pub const OUT_OF_RESOURCES: Status = Status(9 | (1 << (usize::BITS - 1)));
    pub const VOLUME_CORRUPTED: Status = Status(10 | (1 << (usize::BITS - 1)));
    pub const VOLUME_FULL: Status = Status(11 | (1 << (usize::BITS - 1)));
    pub const NO_MEDIA: Status = Status(12 | (1 << (usize::BITS - 1)));
    pub const MEDIA_CHANGED: Status = Status(13 | (1 << (usize::BITS - 1)));
    pub const NOT_FOUND: Status = Status(14 | (1 << (usize::BITS - 1)));
    pub const ACCESS_DENIED: Status = Status(15 | (1 << (usize::BITS - 1)));
    pub const NO_RESPONSE: Status = Status(16 | (1 << (usize::BITS - 1)));
    pub const NO_MAPPING: Status = Status(17 | (1 << (usize::BITS - 1)));
    pub const TIMEOUT: Status = Status(18 | (1 << (usize::BITS - 1)));
    pub const NOT_STARTED: Status = Status(19 | (1 << (usize::BITS - 1)));
    pub const ALREADY_STARTED: Status = Status(20 | (1 << (usize::BITS - 1)));
    pub const ABORTED: Status = Status(21 | (1 << (usize::BITS - 1)));
    pub const ICMP_ERROR: Status = Status(22 | (1 << (usize::BITS - 1)));
    pub const TFTP_ERROR: Status = Status(23 | (1 << (usize::BITS - 1)));
    pub const PROTOCOL_ERROR: Status = Status(24 | (1 << (usize::BITS - 1)));
    pub const INCOMPATIBLE_VERSION: Status = Status(25 | (1 << (usize::BITS - 1)));
    pub const SECURITY_VIOLATION: Status = Status(26 | (1 << (usize::BITS - 1)));
    pub const CRC_ERROR: Status = Status(27 | (1 << (usize::BITS - 1)));
    pub const END_OF_MEDIA: Status = Status(28 | (1 << (usize::BITS - 1)));
    pub const END_OF_FILE: Status = Status(31 | (1 << (usize::BITS - 1)));
    pub const INVALID_LANGUAGE: Status = Status(32 | (1 << (usize::BITS - 1)));
    pub const COMPROMISED_DATA: Status = Status(33 | (1 << (usize::BITS - 1)));

    /// Verifica se o status representa sucesso
    pub fn is_success(&self) -> bool {
        self.0 == 0
    }

    /// Verifica se o status representa um erro
    pub fn is_error(&self) -> bool {
        (self.0 & (1 << (usize::BITS - 1))) != 0
    }

    /// Verifica se o status representa um warning
    pub fn is_warning(&self) -> bool {
        !self.is_error() && self.0 != 0
    }

    /// Converte Status em Result
    pub fn to_result(self) -> Result<()> {
        if self.is_success() { Ok(()) } else { Err(self) }
    }

    /// Converte Status em Result com valor
    pub fn to_result_with_val<T>(self, val: T) -> Result<T> {
        if self.is_success() {
            Ok(val)
        } else {
            Err(self)
        }
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::SUCCESS => write!(f, "SUCCESS"),
            Status::LOAD_ERROR => write!(f, "LOAD_ERROR"),
            Status::INVALID_PARAMETER => write!(f, "INVALID_PARAMETER"),
            Status::UNSUPPORTED => write!(f, "UNSUPPORTED"),
            Status::BAD_BUFFER_SIZE => write!(f, "BAD_BUFFER_SIZE"),
            Status::BUFFER_TOO_SMALL => write!(f, "BUFFER_TOO_SMALL"),
            Status::NOT_READY => write!(f, "NOT_READY"),
            Status::DEVICE_ERROR => write!(f, "DEVICE_ERROR"),
            Status::OUT_OF_RESOURCES => write!(f, "OUT_OF_RESOURCES"),
            Status::NOT_FOUND => write!(f, "NOT_FOUND"),
            Status::ACCESS_DENIED => write!(f, "ACCESS_DENIED"),
            Status::TIMEOUT => write!(f, "TIMEOUT"),
            Status::SECURITY_VIOLATION => write!(f, "SECURITY_VIOLATION"),
            _ => write!(f, "Status({:#x})", self.0),
        }
    }
}

/// Result type para operações UEFI
pub type Result<T> = core::result::Result<T, Status>;

/// GUID - Globally Unique Identifier
///
/// Spec: Appendix A - GUIDs
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Guid {
    /// Cria um novo GUID
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// Tipo booleano UEFI (1 byte)
///
/// Spec: 2.3.1 - Data Types
pub type Boolean = u8;

/// Valor FALSE para Boolean
pub const FALSE: Boolean = 0;

/// Valor TRUE para Boolean
pub const TRUE: Boolean = 1;

/// Char16 - Caractere UCS-2
///
/// Spec: 2.3.1 - Data Types
pub type Char16 = u16;

/// Char8 - Caractere ASCII
pub type Char8 = u8;

/// Event Handle
///
/// Spec: 7.1 - Event, Timer, and Task Priority Services
#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Event(pub *mut core::ffi::c_void);

impl Event {
    pub const fn null() -> Self {
        Event(core::ptr::null_mut())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
