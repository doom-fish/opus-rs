use opus::{Application, Channels, Decoder, Encoder, OpusErrorKind, SampleRate};

const MAX_FRAME: usize = 5760;

fn next_byte(seed: &mut u32) -> u8 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    seed.to_be_bytes()[0]
}

fn assert_bounded(result: opus::Result<usize>) {
    match result {
        Ok(n) => assert!(n <= MAX_FRAME, "decoded {n} samples per channel"),
        Err(e) => assert!(
            matches!(
                e.kind(),
                OpusErrorKind::InvalidPacket
                    | OpusErrorKind::BufferTooSmall
                    | OpusErrorKind::BadArg
            ),
            "unexpected error {e}"
        ),
    }
}

#[test]
fn random_packets_are_rejected_or_decoded_within_bounds() {
    for channels in [Channels::Mono, Channels::Stereo] {
        let mut dec = Decoder::new(SampleRate::Hz48000, channels).expect("decoder");
        let mut out = vec![0i16; MAX_FRAME * channels.count_usize()];
        let mut seed = 0xDEAD_BEEF_u32;
        for len in (1..=96).chain([255, 256, 1275, 1500, 4000]) {
            for _ in 0..40 {
                let packet: Vec<u8> = (0..len).map(|_| next_byte(&mut seed)).collect();
                assert_bounded(dec.decode(&packet, &mut out, false));
            }
        }
    }
}

#[test]
fn truncated_packets_are_handled() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Audio).expect("encoder");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Stereo).expect("decoder");
    let pcm: Vec<i16> = (0..960 * 2)
        .map(|i| i16::try_from(i % 2000).unwrap() - 1000)
        .collect();
    let mut packet = vec![0u8; 4000];
    let n = enc.encode(&pcm, &mut packet).expect("encode");
    let mut out = vec![0i16; MAX_FRAME * 2];
    for end in 1..=n {
        assert_bounded(dec.decode(&packet[..end], &mut out, false));
    }
    let mut corrupted = packet[..n].to_vec();
    let mut seed = 7_u32;
    for _ in 0..500 {
        let i = usize::from(next_byte(&mut seed)) % corrupted.len();
        corrupted[i] ^= next_byte(&mut seed) | 1;
        assert_bounded(dec.decode(&corrupted, &mut out, false));
    }
}
