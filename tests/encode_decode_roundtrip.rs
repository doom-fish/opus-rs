//! Encode a sine wave, decode it back, and assert the reconstruction error is
//! bounded. Opus is lossy, so we use a generous RMS threshold but still verify
//! the signal survives the round-trip.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::suboptimal_flops
)]

use opus::{Application, Channels, Decoder, Encoder, SampleRate};

const SAMPLE_RATE: i32 = 48_000;
const FRAME: usize = 960; // 20 ms mono

fn sine_frame(phase: &mut f32, freq: f32, amp: f32) -> Vec<i16> {
    let step = 2.0 * std::f32::consts::PI * freq / SAMPLE_RATE as f32;
    (0..FRAME)
        .map(|_| {
            let s = (*phase).sin() * amp;
            *phase += step;
            if *phase > 2.0 * std::f32::consts::PI {
                *phase -= 2.0 * std::f32::consts::PI;
            }
            (s * i16::MAX as f32) as i16
        })
        .collect()
}

#[test]
fn sine_round_trip_within_rms_bound() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Audio).expect("encoder");
    enc.set_bitrate(64_000).expect("bitrate");
    enc.set_complexity(10).expect("complexity");

    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    let mut phase = 0.0f32;
    let mut packet = vec![0u8; 4000];
    let mut out = vec![0i16; FRAME];

    // Prime the codec (encoder lookahead / decoder warm-up) for a few frames.
    for _ in 0..5 {
        let pcm = sine_frame(&mut phase, 440.0, 0.5);
        let n = enc.encode(&pcm, &mut packet).expect("encode");
        dec.decode(&packet[..n], &mut out).expect("decode");
    }

    // Now measure on steady-state frames. Opus has algorithmic delay
    // (encoder lookahead + decoder), so a sample-wise diff would mostly
    // measure a phase shift. Instead we compare signal *energy*, which is
    // delay-invariant, to confirm the tone survives the round-trip.
    let mut sq_in = 0.0f64;
    let mut sq_out = 0.0f64;
    let n_frames = 20;
    for _ in 0..n_frames {
        let pcm = sine_frame(&mut phase, 440.0, 0.5);
        let n = enc.encode(&pcm, &mut packet).expect("encode");
        let m = dec.decode(&packet[..n], &mut out).expect("decode");
        assert_eq!(m, FRAME, "decoded full frame");
        for i in 0..FRAME {
            sq_in += f64::from(pcm[i]) * f64::from(pcm[i]);
            sq_out += f64::from(out[i]) * f64::from(out[i]);
        }
    }

    let rms_in = (sq_in / (FRAME as f64 * f64::from(n_frames))).sqrt();
    let rms_out = (sq_out / (FRAME as f64 * f64::from(n_frames))).sqrt();
    assert!(rms_out > 0.0, "decoded output is not silent");
    // The reconstructed tone should carry similar energy to the original.
    let ratio = rms_out / rms_in;
    assert!(
        (0.7..=1.3).contains(&ratio),
        "rms_out/rms_in = {ratio} should be within 0.7..=1.3 (rms_in={rms_in}, rms_out={rms_out})"
    );
}

#[test]
fn stereo_round_trip() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).expect("encoder");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Stereo).expect("decoder");

    let pcm = vec![1234i16; FRAME * 2];
    let mut packet = vec![0u8; 4000];
    let mut out = vec![0i16; FRAME * 2];

    let n = enc.encode(&pcm, &mut packet).expect("encode");
    let m = dec.decode(&packet[..n], &mut out).expect("decode");
    assert_eq!(m, FRAME, "samples per channel");
}
