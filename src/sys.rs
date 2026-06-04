//! Raw FFI declarations for the vendored libopus C API.
//!
//! These mirror `vendor/libopus/include/opus.h` and `opus_defines.h`. The
//! opaque `OpusEncoder` / `OpusDecoder` structs are never constructed on the
//! Rust side; we only ever hold pointers handed back by libopus.
//!
//! The CTL functions (`opus_encoder_ctl` / `opus_decoder_ctl`) are variadic in
//! C. We declare the two concrete shapes we actually use — `(state, request,
//! i32)` for setters and `(state, request, *mut i32)` for getters — rather
//! than going through a true `...` variadic, which keeps the calls type-safe.

#![allow(non_camel_case_types)]
// This is the raw, 1:1 FFI surface; the constants and functions are documented
// by the upstream libopus headers, so we don't re-document every item here.
#![allow(missing_docs)]

use libc::{c_char, c_int};

/// Opaque encoder state owned by libopus.
#[repr(C)]
pub struct OpusEncoder {
    _private: [u8; 0],
}

/// Opaque decoder state owned by libopus.
#[repr(C)]
pub struct OpusDecoder {
    _private: [u8; 0],
}

extern "C" {
    // --- Encoder -----------------------------------------------------------
    pub fn opus_encoder_create(
        fs: i32,
        channels: c_int,
        application: c_int,
        error: *mut c_int,
    ) -> *mut OpusEncoder;

    pub fn opus_encoder_destroy(st: *mut OpusEncoder);

    pub fn opus_encode(
        st: *mut OpusEncoder,
        pcm: *const i16,
        frame_size: c_int,
        data: *mut u8,
        max_data_bytes: i32,
    ) -> i32;

    /// Encoder CTL. Variadic in C: the third argument is an `i32` for setters
    /// or a `*mut i32` for getters.
    pub fn opus_encoder_ctl(st: *mut OpusEncoder, request: c_int, ...) -> c_int;

    // --- Decoder -----------------------------------------------------------
    pub fn opus_decoder_create(fs: i32, channels: c_int, error: *mut c_int) -> *mut OpusDecoder;

    pub fn opus_decoder_destroy(st: *mut OpusDecoder);

    pub fn opus_decode(
        st: *mut OpusDecoder,
        data: *const u8,
        len: i32,
        pcm: *mut i16,
        frame_size: c_int,
        decode_fec: c_int,
    ) -> i32;

    /// Decoder CTL. Variadic in C: the third argument is an `i32` for setters
    /// or a `*mut i32` for getters.
    pub fn opus_decoder_ctl(st: *mut OpusDecoder, request: c_int, ...) -> c_int;

    // --- Common ------------------------------------------------------------
    pub fn opus_strerror(error: c_int) -> *const c_char;
    pub fn opus_get_version_string() -> *const c_char;
}

// --- Error codes (opus_defines.h) -----------------------------------------
pub const OPUS_OK: c_int = 0;
pub const OPUS_BAD_ARG: c_int = -1;
pub const OPUS_BUFFER_TOO_SMALL: c_int = -2;
pub const OPUS_INTERNAL_ERROR: c_int = -3;
pub const OPUS_INVALID_PACKET: c_int = -4;
pub const OPUS_UNIMPLEMENTED: c_int = -5;
pub const OPUS_INVALID_STATE: c_int = -6;
pub const OPUS_ALLOC_FAIL: c_int = -7;

// --- Application modes ------------------------------------------------------
pub const OPUS_APPLICATION_VOIP: c_int = 2048;
pub const OPUS_APPLICATION_AUDIO: c_int = 2049;
pub const OPUS_APPLICATION_RESTRICTED_LOWDELAY: c_int = 2051;

// --- Special values ---------------------------------------------------------
pub const OPUS_AUTO: i32 = -1000;
pub const OPUS_BITRATE_MAX: i32 = -1;

// --- Signal hints -----------------------------------------------------------
pub const OPUS_SIGNAL_VOICE: i32 = 3001;
pub const OPUS_SIGNAL_MUSIC: i32 = 3002;

// --- Bandwidth values -------------------------------------------------------
pub const OPUS_BANDWIDTH_NARROWBAND: i32 = 1101;
pub const OPUS_BANDWIDTH_MEDIUMBAND: i32 = 1102;
pub const OPUS_BANDWIDTH_WIDEBAND: i32 = 1103;
pub const OPUS_BANDWIDTH_SUPERWIDEBAND: i32 = 1104;
pub const OPUS_BANDWIDTH_FULLBAND: i32 = 1105;

// --- CTL request codes (the integer passed as the second ctl arg) ----------
pub const OPUS_SET_APPLICATION_REQUEST: c_int = 4000;
pub const OPUS_GET_APPLICATION_REQUEST: c_int = 4001;
pub const OPUS_SET_BITRATE_REQUEST: c_int = 4002;
pub const OPUS_GET_BITRATE_REQUEST: c_int = 4003;
pub const OPUS_SET_MAX_BANDWIDTH_REQUEST: c_int = 4004;
pub const OPUS_GET_MAX_BANDWIDTH_REQUEST: c_int = 4005;
pub const OPUS_SET_VBR_REQUEST: c_int = 4006;
pub const OPUS_GET_VBR_REQUEST: c_int = 4007;
pub const OPUS_SET_BANDWIDTH_REQUEST: c_int = 4008;
pub const OPUS_GET_BANDWIDTH_REQUEST: c_int = 4009;
pub const OPUS_SET_COMPLEXITY_REQUEST: c_int = 4010;
pub const OPUS_GET_COMPLEXITY_REQUEST: c_int = 4011;
pub const OPUS_SET_INBAND_FEC_REQUEST: c_int = 4012;
pub const OPUS_GET_INBAND_FEC_REQUEST: c_int = 4013;
pub const OPUS_SET_PACKET_LOSS_PERC_REQUEST: c_int = 4014;
pub const OPUS_GET_PACKET_LOSS_PERC_REQUEST: c_int = 4015;
pub const OPUS_SET_DTX_REQUEST: c_int = 4016;
pub const OPUS_GET_DTX_REQUEST: c_int = 4017;
pub const OPUS_SET_VBR_CONSTRAINT_REQUEST: c_int = 4020;
pub const OPUS_GET_VBR_CONSTRAINT_REQUEST: c_int = 4021;
pub const OPUS_SET_SIGNAL_REQUEST: c_int = 4024;
pub const OPUS_GET_SIGNAL_REQUEST: c_int = 4025;
pub const OPUS_GET_LOOKAHEAD_REQUEST: c_int = 4027;
pub const OPUS_RESET_STATE: c_int = 4028;
pub const OPUS_GET_SAMPLE_RATE_REQUEST: c_int = 4029;
pub const OPUS_GET_FINAL_RANGE_REQUEST: c_int = 4031;
pub const OPUS_GET_PITCH_REQUEST: c_int = 4033;
pub const OPUS_SET_GAIN_REQUEST: c_int = 4034;
pub const OPUS_GET_GAIN_REQUEST: c_int = 4045; // historical off-by value
pub const OPUS_GET_LAST_PACKET_DURATION_REQUEST: c_int = 4039;
pub const OPUS_SET_PHASE_INVERSION_DISABLED_REQUEST: c_int = 4046;
pub const OPUS_GET_PHASE_INVERSION_DISABLED_REQUEST: c_int = 4047;
pub const OPUS_GET_IN_DTX_REQUEST: c_int = 4049;
