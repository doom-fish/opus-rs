//! The safe [`Encoder`] wrapper around `OpusEncoder`.

use crate::error::{check, check_count, len_to_c_int, OpusError, Result};
use crate::sys;
use crate::types::{Application, Bandwidth, Bitrate, Channels, SampleRate, Signal};
use core::ptr::NonNull;
use libc::c_int;

/// An Opus encoder.
///
/// Owns the underlying `OpusEncoder` and frees it on drop. Encoding consumes
/// interleaved `i16` PCM and produces Opus packets.
///
/// # Example
/// ```no_run
/// use opus::{Encoder, Application, Channels, SampleRate};
/// let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).unwrap();
/// enc.set_bitrate(32_000).unwrap();
/// let pcm = vec![0i16; 960]; // 20 ms @ 48 kHz mono
/// let mut packet = vec![0u8; 1500];
/// let n = enc.encode(&pcm, &mut packet).unwrap();
/// let _payload = &packet[..n];
/// ```
pub struct Encoder {
    raw: NonNull<sys::OpusEncoder>,
    channels: Channels,
}

// SAFETY: an OpusEncoder is a self-contained heap allocation with no shared
// global state; libopus is safe to move between threads as long as access is
// not concurrent (enforced by `&mut self` on every mutating method).
unsafe impl Send for Encoder {}

impl Encoder {
    /// Create a new encoder for the given sample rate, channel layout and
    /// application mode.
    ///
    /// # Errors
    /// Returns an [`OpusError`] if libopus rejects the parameters or fails to
    /// allocate.
    pub fn new(
        sample_rate: SampleRate,
        channels: Channels,
        application: Application,
    ) -> Result<Self> {
        let mut error: c_int = sys::OPUS_OK;
        // SAFETY: out-pointer is valid; sample rate/channels are validated by
        // the type system to be legal Opus values.
        let raw = unsafe {
            sys::opus_encoder_create(
                sample_rate.hz(),
                channels.count(),
                application.as_raw(),
                &raw mut error,
            )
        };
        if error != sys::OPUS_OK || raw.is_null() {
            return Err(OpusError::from_code(error));
        }
        Ok(Self {
            // SAFETY: checked non-null above.
            raw: unsafe { NonNull::new_unchecked(raw) },
            channels,
        })
    }

    /// Encode one frame of interleaved `i16` PCM into `output`.
    ///
    /// `pcm.len()` must be `frame_size * channels`, where `frame_size` is one
    /// of the valid Opus durations (2.5–120 ms). Returns the number of bytes
    /// written to `output` (the compressed packet length).
    ///
    /// # Errors
    /// Returns an [`OpusError`] if the frame size is invalid or the output
    /// buffer is too small.
    pub fn encode(&mut self, pcm: &[i16], output: &mut [u8]) -> Result<usize> {
        let frame_size = len_to_c_int(pcm.len() / self.channels.count_usize())?;
        let max_bytes = len_to_c_int(output.len())?;
        // SAFETY: pointers/lengths derive from the slices; libopus reads
        // frame_size*channels samples and writes at most max_data_bytes.
        let n = unsafe {
            sys::opus_encode(
                self.raw.as_ptr(),
                pcm.as_ptr(),
                frame_size,
                output.as_mut_ptr(),
                max_bytes,
            )
        };
        check_count(n)
    }

    /// The channel layout this encoder was created with.
    #[must_use]
    pub const fn channels(&self) -> Channels {
        self.channels
    }

    // --- CTL setters -------------------------------------------------------

    fn ctl_set(&mut self, request: c_int, value: i32) -> Result<()> {
        // SAFETY: request/value are valid for the encoder ctl ABI.
        check(unsafe { sys::opus_encoder_ctl(self.raw.as_ptr(), request, value) })?;
        Ok(())
    }

    fn ctl_get(&self, request: c_int) -> Result<i32> {
        let mut value: i32 = 0;
        // SAFETY: out-pointer is valid for the duration of the call.
        check(unsafe { sys::opus_encoder_ctl(self.raw.as_ptr(), request, &mut value) })?;
        Ok(value)
    }

    /// Set the target bitrate in bits per second.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on an out-of-range value.
    pub fn set_bitrate(&mut self, bits_per_second: i32) -> Result<()> {
        self.set_bitrate_mode(Bitrate::BitsPerSecond(bits_per_second))
    }

    /// Set the bitrate using the richer [`Bitrate`] enum (auto / max / fixed).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on an out-of-range value.
    pub fn set_bitrate_mode(&mut self, bitrate: Bitrate) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_BITRATE_REQUEST, bitrate.as_raw())
    }

    /// Get the current bitrate setting.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn bitrate(&self) -> Result<Bitrate> {
        Ok(Bitrate::from_raw(self.ctl_get(sys::OPUS_GET_BITRATE_REQUEST)?))
    }

    /// Set encoder computational complexity (0–10; higher is better quality).
    ///
    /// # Errors
    /// Returns an [`OpusError`] if the value is outside 0–10.
    pub fn set_complexity(&mut self, complexity: i32) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_COMPLEXITY_REQUEST, complexity)
    }

    /// Get the current complexity.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn complexity(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_COMPLEXITY_REQUEST)
    }

    /// Enable or disable variable bitrate (VBR). Enabled by default.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_vbr(&mut self, enabled: bool) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_VBR_REQUEST, i32::from(enabled))
    }

    /// Get whether VBR is enabled.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn vbr(&self) -> Result<bool> {
        Ok(self.ctl_get(sys::OPUS_GET_VBR_REQUEST)? != 0)
    }

    /// Enable or disable constrained VBR.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_vbr_constraint(&mut self, enabled: bool) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_VBR_CONSTRAINT_REQUEST, i32::from(enabled))
    }

    /// Get whether constrained VBR is enabled.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn vbr_constraint(&self) -> Result<bool> {
        Ok(self.ctl_get(sys::OPUS_GET_VBR_CONSTRAINT_REQUEST)? != 0)
    }

    /// Enable or disable in-band forward error correction (FEC).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_inband_fec(&mut self, enabled: bool) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_INBAND_FEC_REQUEST, i32::from(enabled))
    }

    /// Get whether in-band FEC is enabled.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn inband_fec(&self) -> Result<bool> {
        Ok(self.ctl_get(sys::OPUS_GET_INBAND_FEC_REQUEST)? != 0)
    }

    /// Set the expected packet-loss percentage (0–100), which tunes FEC and
    /// redundancy.
    ///
    /// # Errors
    /// Returns an [`OpusError`] if the value is outside 0–100.
    pub fn set_packet_loss_perc(&mut self, perc: i32) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_PACKET_LOSS_PERC_REQUEST, perc)
    }

    /// Get the configured packet-loss percentage.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn packet_loss_perc(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_PACKET_LOSS_PERC_REQUEST)
    }

    /// Enable or disable discontinuous transmission (DTX).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_dtx(&mut self, enabled: bool) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_DTX_REQUEST, i32::from(enabled))
    }

    /// Get whether DTX is enabled.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn dtx(&self) -> Result<bool> {
        Ok(self.ctl_get(sys::OPUS_GET_DTX_REQUEST)? != 0)
    }

    /// Set the maximum encoded audio bandwidth.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_max_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_MAX_BANDWIDTH_REQUEST, bandwidth.as_raw())
    }

    /// Get the configured maximum bandwidth.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure, or if the reported value is
    /// unrecognised.
    pub fn max_bandwidth(&self) -> Result<Bandwidth> {
        let raw = self.ctl_get(sys::OPUS_GET_MAX_BANDWIDTH_REQUEST)?;
        Bandwidth::from_raw(raw).ok_or_else(|| OpusError::from_code(sys::OPUS_INVALID_STATE))
    }

    /// Force a specific encoded bandwidth.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_bandwidth(&mut self, bandwidth: Bandwidth) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_BANDWIDTH_REQUEST, bandwidth.as_raw())
    }

    /// Set the application/tuning mode.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_application(&mut self, application: Application) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_APPLICATION_REQUEST, application.as_raw())
    }

    /// Get the current application/tuning mode.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure, or if the reported value is
    /// unrecognised.
    pub fn application(&self) -> Result<Application> {
        let raw = self.ctl_get(sys::OPUS_GET_APPLICATION_REQUEST)?;
        Application::from_raw(raw).ok_or_else(|| OpusError::from_code(sys::OPUS_INVALID_STATE))
    }

    /// Set the signal hint (auto / voice / music).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_signal(&mut self, signal: Signal) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_SIGNAL_REQUEST, signal.as_raw())
    }

    /// Get the current signal hint.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure, or if the reported value is
    /// unrecognised.
    pub fn signal(&self) -> Result<Signal> {
        let raw = self.ctl_get(sys::OPUS_GET_SIGNAL_REQUEST)?;
        Signal::from_raw(raw).ok_or_else(|| OpusError::from_code(sys::OPUS_INVALID_STATE))
    }

    /// Get the encoder lookahead in samples (the algorithmic delay).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn lookahead(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_LOOKAHEAD_REQUEST)
    }

    /// Get the sample rate the encoder was created with.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn sample_rate(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_SAMPLE_RATE_REQUEST)
    }

    /// Reset the encoder to a freshly-initialised state, keeping its settings.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn reset_state(&mut self) -> Result<()> {
        // OPUS_RESET_STATE takes no argument; the value is ignored.
        self.ctl_set(sys::OPUS_RESET_STATE, 0)
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        // SAFETY: raw was created by opus_encoder_create and not yet freed.
        unsafe { sys::opus_encoder_destroy(self.raw.as_ptr()) }
    }
}
