//! Error handling mapped from libopus return codes.

use crate::sys;
use core::ffi::CStr;
use core::fmt;
use libc::c_int;

/// An error returned by libopus, mapped from a negative C return code.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OpusError {
    code: c_int,
}

impl OpusError {
    /// Wrap a raw libopus error code.
    #[must_use]
    pub const fn from_code(code: c_int) -> Self {
        Self { code }
    }

    /// The raw libopus error code (always negative for real errors).
    #[must_use]
    pub const fn code(self) -> i32 {
        self.code
    }

    /// A stable, human-readable kind for this error.
    #[must_use]
    pub const fn kind(self) -> OpusErrorKind {
        match self.code {
            sys::OPUS_BAD_ARG => OpusErrorKind::BadArg,
            sys::OPUS_BUFFER_TOO_SMALL => OpusErrorKind::BufferTooSmall,
            sys::OPUS_INTERNAL_ERROR => OpusErrorKind::InternalError,
            sys::OPUS_INVALID_PACKET => OpusErrorKind::InvalidPacket,
            sys::OPUS_UNIMPLEMENTED => OpusErrorKind::Unimplemented,
            sys::OPUS_INVALID_STATE => OpusErrorKind::InvalidState,
            sys::OPUS_ALLOC_FAIL => OpusErrorKind::AllocFail,
            _ => OpusErrorKind::Unknown,
        }
    }

    /// The description string from `opus_strerror`.
    #[must_use]
    pub fn message(self) -> &'static str {
        // SAFETY: opus_strerror always returns a valid, static, NUL-terminated
        // C string for any input.
        unsafe {
            let ptr = sys::opus_strerror(self.code);
            CStr::from_ptr(ptr).to_str().unwrap_or("invalid opus error")
        }
    }
}

/// A coarse, matchable classification of an [`OpusError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OpusErrorKind {
    /// One or more invalid arguments.
    BadArg,
    /// The supplied buffer was too small.
    BufferTooSmall,
    /// An internal libopus error.
    InternalError,
    /// The compressed packet passed in was corrupt.
    InvalidPacket,
    /// A requested action is not implemented.
    Unimplemented,
    /// The encoder/decoder structure was in an invalid state.
    InvalidState,
    /// Memory allocation failed.
    AllocFail,
    /// An unrecognised error code.
    Unknown,
}

impl fmt::Display for OpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "opus error {}: {}", self.code, self.message())
    }
}

impl fmt::Debug for OpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpusError")
            .field("code", &self.code)
            .field("kind", &self.kind())
            .field("message", &self.message())
            .finish()
    }
}

impl std::error::Error for OpusError {}

/// Convenience result type for fallible opus operations.
pub type Result<T> = core::result::Result<T, OpusError>;

/// Turn a libopus return value into a `Result`: negative values are errors,
/// non-negative values are returned verbatim.
pub const fn check(code: c_int) -> Result<c_int> {
    if code < 0 {
        Err(OpusError::from_code(code))
    } else {
        Ok(code)
    }
}

/// Like [`check`], but converts a non-negative count to `usize`.
pub fn check_count(code: c_int) -> Result<usize> {
    let n = check(code)?;
    Ok(usize::try_from(n).unwrap_or(0))
}

/// Convert a slice length to the `c_int` libopus expects, rejecting overflow.
pub fn len_to_c_int(len: usize) -> Result<c_int> {
    c_int::try_from(len).map_err(|_| OpusError::from_code(crate::sys::OPUS_BAD_ARG))
}
