//! Strongly-typed enums for the libopus configuration surface.

use crate::sys;
use libc::c_int;

/// Sample rates supported by Opus, in Hz.
///
/// These are the only rates an Opus encoder/decoder may be created with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SampleRate {
    /// 8 kHz (narrowband).
    Hz8000,
    /// 12 kHz (mediumband).
    Hz12000,
    /// 16 kHz (wideband).
    Hz16000,
    /// 24 kHz (super-wideband).
    Hz24000,
    /// 48 kHz (fullband) — the canonical WebRTC rate.
    Hz48000,
}

impl SampleRate {
    /// The numeric sample rate in Hz.
    #[must_use]
    pub const fn hz(self) -> i32 {
        match self {
            Self::Hz8000 => 8000,
            Self::Hz12000 => 12000,
            Self::Hz16000 => 16000,
            Self::Hz24000 => 24000,
            Self::Hz48000 => 48000,
        }
    }

    /// Build a `SampleRate` from a raw Hz value, if it is a valid Opus rate.
    #[must_use]
    pub const fn from_hz(hz: i32) -> Option<Self> {
        match hz {
            8000 => Some(Self::Hz8000),
            12000 => Some(Self::Hz12000),
            16000 => Some(Self::Hz16000),
            24000 => Some(Self::Hz24000),
            48000 => Some(Self::Hz48000),
            _ => None,
        }
    }
}

/// Channel layout. Opus supports mono and stereo for the basic API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channels {
    /// A single channel.
    Mono,
    /// Two interleaved channels (L, R).
    Stereo,
}

impl Channels {
    /// The channel count as an integer.
    #[must_use]
    pub const fn count(self) -> i32 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }

    /// The channel count as a `usize` (for slice arithmetic).
    #[must_use]
    pub const fn count_usize(self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }
}

/// Encoder application / tuning mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Application {
    /// Best for most `VoIP` / videoconference speech. Favours intelligibility.
    Voip,
    /// Best for broadcast / high-fidelity music.
    Audio,
    /// Lowest-delay mode; disables the speech-optimised SILK layer.
    RestrictedLowDelay,
}

impl Application {
    pub(crate) const fn as_raw(self) -> c_int {
        match self {
            Self::Voip => sys::OPUS_APPLICATION_VOIP,
            Self::Audio => sys::OPUS_APPLICATION_AUDIO,
            Self::RestrictedLowDelay => sys::OPUS_APPLICATION_RESTRICTED_LOWDELAY,
        }
    }

    pub(crate) const fn from_raw(raw: c_int) -> Option<Self> {
        match raw {
            sys::OPUS_APPLICATION_VOIP => Some(Self::Voip),
            sys::OPUS_APPLICATION_AUDIO => Some(Self::Audio),
            sys::OPUS_APPLICATION_RESTRICTED_LOWDELAY => Some(Self::RestrictedLowDelay),
            _ => None,
        }
    }
}

/// Audio bandwidth, used with the encoder's max-bandwidth CTL and reported by
/// the decoder's bandwidth CTL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bandwidth {
    /// 4 kHz bandpass.
    Narrowband,
    /// 6 kHz bandpass.
    Mediumband,
    /// 8 kHz bandpass.
    Wideband,
    /// 12 kHz bandpass.
    SuperWideband,
    /// 20 kHz bandpass (full range).
    Fullband,
}

impl Bandwidth {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::Narrowband => sys::OPUS_BANDWIDTH_NARROWBAND,
            Self::Mediumband => sys::OPUS_BANDWIDTH_MEDIUMBAND,
            Self::Wideband => sys::OPUS_BANDWIDTH_WIDEBAND,
            Self::SuperWideband => sys::OPUS_BANDWIDTH_SUPERWIDEBAND,
            Self::Fullband => sys::OPUS_BANDWIDTH_FULLBAND,
        }
    }

    pub(crate) const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            sys::OPUS_BANDWIDTH_NARROWBAND => Some(Self::Narrowband),
            sys::OPUS_BANDWIDTH_MEDIUMBAND => Some(Self::Mediumband),
            sys::OPUS_BANDWIDTH_WIDEBAND => Some(Self::Wideband),
            sys::OPUS_BANDWIDTH_SUPERWIDEBAND => Some(Self::SuperWideband),
            sys::OPUS_BANDWIDTH_FULLBAND => Some(Self::Fullband),
            _ => None,
        }
    }
}

/// The kind of signal being encoded — a hint that biases the encoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Signal {
    /// Automatic detection (the default).
    Auto,
    /// Bias toward speech.
    Voice,
    /// Bias toward music.
    Music,
}

impl Signal {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::Auto => sys::OPUS_AUTO,
            Self::Voice => sys::OPUS_SIGNAL_VOICE,
            Self::Music => sys::OPUS_SIGNAL_MUSIC,
        }
    }

    pub(crate) const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            sys::OPUS_AUTO => Some(Self::Auto),
            sys::OPUS_SIGNAL_VOICE => Some(Self::Voice),
            sys::OPUS_SIGNAL_MUSIC => Some(Self::Music),
            _ => None,
        }
    }
}

/// A target bitrate for the encoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bitrate {
    /// A specific bitrate in bits per second.
    BitsPerSecond(i32),
    /// Let the encoder pick based on signal/bandwidth/channels.
    Auto,
    /// Use as many bits as the frame allows.
    Max,
}

impl Bitrate {
    pub(crate) const fn as_raw(self) -> i32 {
        match self {
            Self::BitsPerSecond(b) => b,
            Self::Auto => sys::OPUS_AUTO,
            Self::Max => sys::OPUS_BITRATE_MAX,
        }
    }

    pub(crate) const fn from_raw(raw: i32) -> Self {
        match raw {
            sys::OPUS_AUTO => Self::Auto,
            sys::OPUS_BITRATE_MAX => Self::Max,
            b => Self::BitsPerSecond(b),
        }
    }
}
