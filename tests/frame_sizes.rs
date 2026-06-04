//! Verify every valid Opus frame size encodes and decodes at 48 kHz mono.
//!
//! Valid durations are 2.5, 5, 10, 20, 40, 60, 80, 100, 120 ms.

use opus::{Application, Channels, Decoder, Encoder, SampleRate};

/// Frame durations in milliseconds and the corresponding mono sample counts at
/// 48 kHz.
const FRAMES_48K: &[(f32, usize)] = &[
    (2.5, 120),
    (5.0, 240),
    (10.0, 480),
    (20.0, 960),
    (40.0, 1920),
    (60.0, 2880),
    (80.0, 3840),
    (100.0, 4800),
    (120.0, 5760),
];

#[test]
fn all_frame_sizes_48k_mono() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    let mut packet = vec![0u8; 8000];

    for &(ms, samples) in FRAMES_48K {
        let pcm = vec![0i16; samples];
        let n = enc
            .encode(&pcm, &mut packet)
            .unwrap_or_else(|e| panic!("encode {ms}ms ({samples} samples): {e}"));
        assert!(n > 0, "{ms}ms produced a non-empty packet");

        let mut out = vec![0i16; samples];
        let m = dec
            .decode(&packet[..n], &mut out, false)
            .unwrap_or_else(|e| panic!("decode {ms}ms: {e}"));
        assert_eq!(m, samples, "{ms}ms decoded sample count");
    }
}

#[test]
fn frame_sizes_across_sample_rates() {
    // 20 ms frames at each supported rate.
    let rates = [
        (SampleRate::Hz8000, 160),
        (SampleRate::Hz12000, 240),
        (SampleRate::Hz16000, 320),
        (SampleRate::Hz24000, 480),
        (SampleRate::Hz48000, 960),
    ];

    for (rate, samples) in rates {
        let mut enc = Encoder::new(rate, Channels::Mono, Application::Voip).expect("encoder");
        let mut dec = Decoder::new(rate, Channels::Mono).expect("decoder");

        let pcm = vec![0i16; samples];
        let mut packet = vec![0u8; 4000];
        let n = enc.encode(&pcm, &mut packet).expect("encode");
        let mut out = vec![0i16; samples];
        let m = dec.decode(&packet[..n], &mut out, false).expect("decode");
        assert_eq!(m, samples, "{}Hz 20ms", rate.hz());
    }
}
