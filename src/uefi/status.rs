//! Tratamento de Erros e Status UEFI
//!
//! Fornece uma abstração robusta sobre os códigos de retorno numéricos do UEFI
//! (`usize`). Permite o uso idiomático de `Result<T, Status>` no Rust.
//!
//! Referência: UEFI Spec 2.10, Appendix D - Status Codes

use core::fmt;

/// Resultado padrão para operações UEFI.
pub type Result<T> = core::result::Result<T, Status>;

/// Representa o código de status `EFI_STATUS`.
/// O bit mais significativo define se é erro (1) ou sucesso/aviso (0).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Status(pub usize);

impl Status {
    // --- Sucesso ---
    pub const SUCCESS: Status = Status(0);

    // --- Warnings (Bit alto = 0) ---
    pub const WARN_UNKNOWN_GLYPH: Status = Status(1);
    pub const WARN_DELETE_FAILURE: Status = Status(2);
    pub const WARN_WRITE_FAILURE: Status = Status(3);
    pub const WARN_BUFFER_TOO_SMALL: Status = Status(4);
    pub const WARN_STALE_DATA: Status = Status(5);
    pub const WARN_FILE_SYSTEM: Status = Status(6);
    pub const WARN_RESET_REQUIRED: Status = Status(7);

    // --- Erros (Bit alto = 1) ---
    // A constante ERROR_BIT depende da arquitetura (64-bit = bit 63).
    const ERROR_BIT: usize = 1 << (usize::BITS - 1);

    pub const LOAD_ERROR: Status = Status(Self::ERROR_BIT | 1);
    pub const INVALID_PARAMETER: Status = Status(Self::ERROR_BIT | 2);
    pub const UNSUPPORTED: Status = Status(Self::ERROR_BIT | 3);
    pub const BAD_BUFFER_SIZE: Status = Status(Self::ERROR_BIT | 4);
    pub const BUFFER_TOO_SMALL: Status = Status(Self::ERROR_BIT | 5);
    pub const NOT_READY: Status = Status(Self::ERROR_BIT | 6);
    pub const DEVICE_ERROR: Status = Status(Self::ERROR_BIT | 7);
    pub const WRITE_PROTECTED: Status = Status(Self::ERROR_BIT | 8);
    pub const OUT_OF_RESOURCES: Status = Status(Self::ERROR_BIT | 9);
    pub const VOLUME_CORRUPTED: Status = Status(Self::ERROR_BIT | 10);
    pub const VOLUME_FULL: Status = Status(Self::ERROR_BIT | 11);
    pub const NO_MEDIA: Status = Status(Self::ERROR_BIT | 12);
    pub const MEDIA_CHANGED: Status = Status(Self::ERROR_BIT | 13);
    pub const NOT_FOUND: Status = Status(Self::ERROR_BIT | 14);
    pub const ACCESS_DENIED: Status = Status(Self::ERROR_BIT | 15);
    pub const NO_RESPONSE: Status = Status(Self::ERROR_BIT | 16);
    pub const NO_MAPPING: Status = Status(Self::ERROR_BIT | 17);
    pub const TIMEOUT: Status = Status(Self::ERROR_BIT | 18);
    pub const NOT_STARTED: Status = Status(Self::ERROR_BIT | 19);
    pub const ALREADY_STARTED: Status = Status(Self::ERROR_BIT | 20);
    pub const ABORTED: Status = Status(Self::ERROR_BIT | 21);
    pub const ICMP_ERROR: Status = Status(Self::ERROR_BIT | 22);
    pub const TFTP_ERROR: Status = Status(Self::ERROR_BIT | 23);
    pub const PROTOCOL_ERROR: Status = Status(Self::ERROR_BIT | 24);
    pub const INCOMPATIBLE_VERSION: Status = Status(Self::ERROR_BIT | 25);
    pub const SECURITY_VIOLATION: Status = Status(Self::ERROR_BIT | 26);
    pub const CRC_ERROR: Status = Status(Self::ERROR_BIT | 27);
    pub const END_OF_MEDIA: Status = Status(Self::ERROR_BIT | 28);
    pub const END_OF_FILE: Status = Status(Self::ERROR_BIT | 31);

    /// Retorna `true` se o status indica sucesso.
    #[inline]
    pub fn is_success(self) -> bool {
        self == Status::SUCCESS
    }

    /// Retorna `true` se o status indica um erro (bit alto setado).
    #[inline]
    pub fn is_error(self) -> bool {
        (self.0 & Self::ERROR_BIT) != 0
    }

    /// Helper para converter Status em Result do Rust (vazio).
    #[inline]
    pub fn to_result(self) -> Result<()> {
        if self.is_success() { Ok(()) } else { Err(self) }
    }

    /// Helper para converter Status em Result contendo um valor.
    /// Útil para wrappers seguros que retornam dados.
    #[inline]
    pub fn to_result_with<T>(self, val: T) -> Result<T> {
        if self.is_success() {
            Ok(val)
        } else {
            Err(self)
        }
    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match *self {
            Status::SUCCESS => "SUCCESS",
            Status::WARN_BUFFER_TOO_SMALL => "WARN_BUFFER_TOO_SMALL",
            Status::INVALID_PARAMETER => "INVALID_PARAMETER",
            Status::UNSUPPORTED => "UNSUPPORTED",
            Status::BAD_BUFFER_SIZE => "BAD_BUFFER_SIZE",
            Status::BUFFER_TOO_SMALL => "BUFFER_TOO_SMALL",
            Status::NOT_READY => "NOT_READY",
            Status::DEVICE_ERROR => "DEVICE_ERROR",
            Status::WRITE_PROTECTED => "WRITE_PROTECTED",
            Status::OUT_OF_RESOURCES => "OUT_OF_RESOURCES",
            Status::NOT_FOUND => "NOT_FOUND",
            Status::ACCESS_DENIED => "ACCESS_DENIED",
            Status::TIMEOUT => "TIMEOUT",
            Status::ABORTED => "ABORTED",
            Status::PROTOCOL_ERROR => "PROTOCOL_ERROR",
            Status::END_OF_FILE => "END_OF_FILE",
            _ => "UNKNOWN",
        };

        if self.is_success() {
            write!(f, "{}", name)
        } else {
            write!(f, "{}({:#x})", name, self.0)
        }
    }
}
