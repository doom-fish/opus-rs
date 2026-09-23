//! The safe [`Decoder`] wrapper around `OpusDecoder`.

use crate::error::{check, check_count, len_to_c_int, OpusError, Result};
use crate::sys;
use crate::types::{Bandwidth, Channels, SampleRate};
use core::ptr::NonNull;
use libc::c_int;

/// An Opus decoder.
///
/// Owns the underlying `OpusDecoder` and frees it on drop. Decoding consumes
/// Opus packets and produces interleaved `i16` PCM.
///
/// # Example
/// ```no_run
/// use opus::{Decoder, Channels, SampleRate};
/// let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).unwrap();
/// let packet: &[u8] = &[/* ... */];
/// let mut pcm = vec![0i16; 960]; // 20 ms @ 48 kHz mono
/// let _m = dec.decode(packet, &mut pcm).unwrap();
/// ```
pub struct Decoder {
    raw: NonNull<sys::OpusDecoder>,
    channels: Channels,
    sample_rate: SampleRate,
    last_fec_used: bool,
}

// SAFETY: see the equivalent note on `Encoder` — no shared global state, and
// `&mut self` serialises all mutating access.
unsafe impl Send for Decoder {}

impl Decoder {
    /// Create a new decoder for the given sample rate and channel layout.
    ///
    /// # Errors
    /// Returns an [`OpusError`] if libopus rejects the parameters or fails to
    /// allocate.
    pub fn new(sample_rate: SampleRate, channels: Channels) -> Result<Self> {
        let mut error: c_int = sys::OPUS_OK;
        // SAFETY: out-pointer is valid; rate/channels are type-validated.
        let raw =
            unsafe { sys::opus_decoder_create(sample_rate.hz(), channels.count(), &mut error) };
        if error != sys::OPUS_OK || raw.is_null() {
            return Err(OpusError::from_code(error));
        }
        Ok(Self {
            // SAFETY: checked non-null above.
            raw: unsafe { NonNull::new_unchecked(raw) },
            channels,
            sample_rate,
            last_fec_used: false,
        })
    }

    /// Decode an Opus packet into interleaved `i16` PCM.
    ///
    /// `output` must be large enough for one frame: `frame_size * channels`
    /// samples, where `frame_size` is the duration of the packet. Returns the
    /// number of samples **per channel** decoded.
    ///
    /// # Errors
    /// Returns an [`OpusError`] for a corrupt packet or undersized buffer, and
    /// `BadArg` for an empty packet (use [`decode_lost`](Self::decode_lost)
    /// for a lost packet, since libopus would conceal the whole buffer).
    pub fn decode(&mut self, packet: &[u8], output: &mut [i16]) -> Result<usize> {
        if packet.is_empty() {
            return Err(OpusError::from_code(sys::OPUS_BAD_ARG));
        }
        let frame_size = len_to_c_int(output.len() / self.channels.count_usize())?;
        self.decode_into(packet, output, frame_size, false)
    }

    #[allow(missing_docs, clippy::missing_errors_doc)]
    pub fn decode_fec(
        &mut self,
        packet: &[u8],
        output: &mut [i16],
        frame_size: usize,
    ) -> Result<usize> {
        if packet.is_empty() {
            return Err(OpusError::from_code(sys::OPUS_BAD_ARG));
        }
        let frame_size = self.concealment_frame_size(output.len(), frame_size)?;
        self.decode_into(packet, output, frame_size, true)
    }

    /// Perform packet-loss concealment for a single missing frame.
    ///
    /// Call this when a packet is lost and no FEC is available: libopus
    /// synthesises plausible audio to fill the gap. `frame_size` is the
    /// duration to conceal in samples per channel: a multiple of 2.5 ms up to
    /// 120 ms, normally the duration of the lost packet. `output` must hold
    /// `frame_size * channels` samples; only that prefix is written. Returns
    /// samples per channel.
    ///
    /// # Errors
    /// Returns `BadArg` for an invalid `frame_size`, `BufferTooSmall` if
    /// `output` cannot hold it, or another [`OpusError`] on a decode failure.
    pub fn decode_lost(&mut self, output: &mut [i16], frame_size: usize) -> Result<usize> {
        let frame_size = self.concealment_frame_size(output.len(), frame_size)?;
        self.decode_into(&[], output, frame_size, false)
    }

    #[allow(missing_docs, clippy::missing_errors_doc)]
    pub fn packet_duration(&self, packet: &[u8]) -> Result<usize> {
        let len = len_to_c_int(packet.len())?;
        check_count(unsafe {
            sys::opus_decoder_get_nb_samples(self.raw.as_ptr(), packet.as_ptr(), len)
        })
    }

    fn concealment_frame_size(&self, output_len: usize, frame_size: usize) -> Result<c_int> {
        let quantum = self.sample_rate.samples_per_2_5_ms();
        if frame_size == 0 || frame_size % quantum != 0 || frame_size > quantum * 48 {
            return Err(OpusError::from_code(sys::OPUS_BAD_ARG));
        }
        if output_len < frame_size * self.channels.count_usize() {
            return Err(OpusError::from_code(sys::OPUS_BUFFER_TOO_SMALL));
        }
        len_to_c_int(frame_size)
    }

    fn decode_into(
        &mut self,
        packet: &[u8],
        output: &mut [i16],
        frame_size: c_int,
        fec: bool,
    ) -> Result<usize> {
        let (data, packet_len) = if packet.is_empty() {
            (core::ptr::null(), 0)
        } else {
            (packet.as_ptr(), len_to_c_int(packet.len())?)
        };
        // SAFETY: pointers/lengths derive from the slices; every caller passes
        // a frame_size with frame_size*channels <= output.len(), and libopus
        // writes at most that many samples. A null data pointer with len 0 is
        // the libopus convention for packet-loss concealment.
        let n = unsafe {
            sys::opus_decode(
                self.raw.as_ptr(),
                data,
                packet_len,
                output.as_mut_ptr(),
                frame_size,
                i32::from(fec),
            )
        };
        let samples = check_count(n)?;
        self.last_fec_used = fec;
        Ok(samples)
    }

    /// The channel layout this decoder was created with.
    #[must_use]
    pub const fn channels(&self) -> Channels {
        self.channels
    }

    // --- CTL helpers -------------------------------------------------------

    fn ctl_set(&mut self, request: c_int, value: i32) -> Result<()> {
        // SAFETY: request/value are valid for the decoder ctl ABI.
        check(unsafe { sys::opus_decoder_ctl(self.raw.as_ptr(), request, value) })?;
        Ok(())
    }

    fn ctl_get(&self, request: c_int) -> Result<i32> {
        let mut value: i32 = 0;
        // SAFETY: out-pointer valid for the call.
        check(unsafe { sys::opus_decoder_ctl(self.raw.as_ptr(), request, &mut value) })?;
        Ok(value)
    }

    /// Whether the most recent decode call was a
    /// [`decode_fec`](Self::decode_fec) in-band forward error correction.
    ///
    /// libopus has no dedicated *decoder*-side FEC getter — FEC is an encoder
    /// decision and the decoder transparently consumes it when you call
    /// `decode_fec`. This convenience getter reports whether the last decode
    /// was performed in FEC mode (and is `false` after a
    /// [`decode`](Self::decode) or [`decode_lost`](Self::decode_lost) call, or
    /// before any decode).
    #[must_use]
    pub const fn get_in_band_fec(&self) -> bool {
        self.last_fec_used
    }

    /// Set a decoder output gain in Q8 dB (a fixed-point scale factor).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn set_gain(&mut self, gain_q8: i32) -> Result<()> {
        self.ctl_set(sys::OPUS_SET_GAIN_REQUEST, gain_q8)
    }

    /// Get the configured decoder output gain (Q8 dB).
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn gain(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_GAIN_REQUEST)
    }

    /// Get the audio bandwidth of the last decoded packet.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure, or if the reported value is
    /// unrecognised.
    pub fn bandwidth(&self) -> Result<Bandwidth> {
        let raw = self.ctl_get(sys::OPUS_GET_BANDWIDTH_REQUEST)?;
        Bandwidth::from_raw(raw).ok_or_else(|| OpusError::from_code(sys::OPUS_INVALID_STATE))
    }

    /// Get the duration (in samples) of the last decoded packet.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn last_packet_duration(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_LAST_PACKET_DURATION_REQUEST)
    }

    /// Get the sample rate the decoder was created with.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn sample_rate(&self) -> Result<i32> {
        self.ctl_get(sys::OPUS_GET_SAMPLE_RATE_REQUEST)
    }

    /// Reset the decoder to a freshly-initialised state.
    ///
    /// # Errors
    /// Returns an [`OpusError`] on a ctl failure.
    pub fn reset_state(&mut self) -> Result<()> {
        self.ctl_set(sys::OPUS_RESET_STATE, 0)
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: raw was created by opus_decoder_create and not yet freed.
        unsafe { sys::opus_decoder_destroy(self.raw.as_ptr()) }
    }
}
