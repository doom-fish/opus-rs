//! # opus
//!
//! Safe Rust bindings to [libopus](https://opus-codec.org), the reference
//! implementation of the Opus audio codec, statically linked from a vendored
//! source tree.
//!
//! ## Why this crate exists
//!
//! The doomfish workspace is otherwise a pure-Rust / Apple-SDK binding family.
//! `opus-rs` is the **one explicit C-FFI exception**: no Apple framework ships
//! an Opus codec, and standard WebRTC mandates Opus for audio interop between
//! peers. To keep latency-sensitive real-time audio working across every
//! WebRTC peer, we vendor and statically link libopus.
//!
//! ## How it is built
//!
//! libopus 1.5.2 is vendored under `vendor/libopus/` and compiled by
//! `build.rs`, which invokes `clang` directly through
//! [`std::process::Command`] — there is **no** dependency on `cc`, `bindgen`,
//! `pkg-config`, `cmake` or `autoconf`. The build is **fixed-point**
//! (`-DFIXED_POINT=1`) with all SIMD intrinsics disabled, so the identical
//! portable C compiles on every architecture. The only runtime dependency is
//! `libc`.
//!
//! ## Quick start
//!
//! ```no_run
//! use opus::{Encoder, Decoder, Application, Channels, SampleRate};
//!
//! // 20 ms mono frame at 48 kHz = 960 samples.
//! let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)?;
//! enc.set_bitrate(32_000)?;
//! enc.set_inband_fec(true)?;
//! enc.set_packet_loss_perc(10)?;
//!
//! let pcm_i16_samples = vec![0i16; 960];
//! let mut packet = vec![0u8; 1500];
//! let n = enc.encode(&pcm_i16_samples, &mut packet)?;
//! let payload = &packet[..n];
//!
//! let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono)?;
//! let mut pcm_out = vec![0i16; 960];
//! let _m = dec.decode(payload, &mut pcm_out, /* fec = */ false)?;
//! # Ok::<(), opus::OpusError>(())
//! ```
//!
//! For real-time use, always prefer **20 ms frames** between WebRTC peers.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod decoder;
mod encoder;
mod error;
mod types;

pub mod sys;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use error::{OpusError, OpusErrorKind, Result};
pub use types::{Application, Bandwidth, Bitrate, Channels, SampleRate, Signal};

use core::ffi::CStr;

/// The libopus version string, e.g. `"libopus 1.5.2"`.
///
/// This reflects the vendored, statically-linked library.
#[must_use]
pub fn version() -> &'static str {
    // SAFETY: opus_get_version_string returns a valid static C string.
    unsafe {
        CStr::from_ptr(sys::opus_get_version_string())
            .to_str()
            .unwrap_or("libopus (unknown version)")
    }
}
