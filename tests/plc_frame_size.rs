use opus::{Application, Channels, Decoder, Encoder, OpusErrorKind, SampleRate};

const SENTINEL: i16 = 0x5A5A;

fn encode(enc: &mut Encoder, samples: usize, channels: usize) -> Vec<u8> {
    let pcm: Vec<i16> = (0..samples * channels)
        .map(|i| i16::try_from(i % 3000).unwrap() - 1500)
        .collect();
    let mut packet = vec![0u8; 4000];
    let n = enc.encode(&pcm, &mut packet).expect("encode");
    packet.truncate(n);
    packet
}

fn kind(result: opus::Result<usize>) -> OpusErrorKind {
    result.expect_err("expected an error").kind()
}

#[test]
fn concealment_covers_only_the_requested_duration() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");
    let packet = encode(&mut enc, 960, 1);
    let mut out = vec![0i16; 5760];
    assert_eq!(dec.decode(&packet, &mut out).unwrap(), 960);

    out.fill(SENTINEL);
    assert_eq!(dec.decode_lost(&mut out, 960).unwrap(), 960);
    assert!(
        out[960..].iter().all(|&s| s == SENTINEL),
        "concealment wrote past the requested 20 ms"
    );
    assert_eq!(dec.last_packet_duration().unwrap(), 960);
    assert!(!dec.get_in_band_fec());
}

#[test]
fn fec_uses_the_explicit_frame_size() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    enc.set_bitrate(24_000).expect("bitrate");
    enc.set_inband_fec(true).expect("fec");
    enc.set_packet_loss_perc(30).expect("loss");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    let first = encode(&mut enc, 960, 1);
    let second = encode(&mut enc, 960, 1);
    let mut out = vec![0i16; 5760];
    dec.decode(&first, &mut out).expect("decode first");

    out.fill(SENTINEL);
    assert_eq!(dec.decode_fec(&second, &mut out, 960).unwrap(), 960);
    assert!(out[960..].iter().all(|&s| s == SENTINEL));
    assert!(dec.get_in_band_fec());

    assert_eq!(dec.decode(&second, &mut out).unwrap(), 960);
    assert!(!dec.get_in_band_fec());
}

#[test]
fn invalid_concealment_frame_sizes_are_rejected() {
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Stereo).expect("decoder");
    let mut out = vec![0i16; 5760 * 2 + 240];
    for bad in [0, 1, 119, 959, 5761, 5880, usize::MAX] {
        assert_eq!(
            kind(dec.decode_lost(&mut out, bad)),
            OpusErrorKind::BadArg,
            "{bad}"
        );
        assert_eq!(
            kind(dec.decode_fec(&[0xF8, 0xFF, 0xFE], &mut out, bad)),
            OpusErrorKind::BadArg,
            "{bad}"
        );
    }
    assert_eq!(
        kind(dec.decode_lost(&mut out[..959], 480)),
        OpusErrorKind::BufferTooSmall
    );
    assert_eq!(dec.decode_lost(&mut out[..960], 480).unwrap(), 480);
    for ok in [120, 240, 480, 960, 1920, 2880, 3840, 4800, 5760] {
        assert_eq!(dec.decode_lost(&mut out, ok).unwrap(), ok);
    }
}

#[test]
fn concealment_quantum_follows_the_sample_rate() {
    let mut dec = Decoder::new(SampleRate::Hz16000, Channels::Mono).expect("decoder");
    let mut out = vec![0i16; 1920];
    assert_eq!(dec.decode_lost(&mut out, 40).unwrap(), 40);
    assert_eq!(dec.decode_lost(&mut out, 320).unwrap(), 320);
    assert_eq!(dec.decode_lost(&mut out, 1920).unwrap(), 1920);
    assert_eq!(kind(dec.decode_lost(&mut out, 60)), OpusErrorKind::BadArg);
    assert_eq!(kind(dec.decode_lost(&mut out, 1960)), OpusErrorKind::BadArg);
}

#[test]
fn packet_duration_reports_samples_per_channel() {
    let dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");
    for samples in [120, 480, 960, 2880] {
        let mut enc =
            Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Audio).expect("encoder");
        let packet = encode(&mut enc, samples, 1);
        assert_eq!(dec.packet_duration(&packet).unwrap(), samples);
    }

    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    let twenty_ms = encode(&mut enc, 960, 1);
    let dec16 = Decoder::new(SampleRate::Hz16000, Channels::Mono).expect("decoder");
    assert_eq!(dec16.packet_duration(&twenty_ms).unwrap(), 320);

    assert!(dec.packet_duration(&[]).is_err());
    assert_eq!(
        kind(dec.packet_duration(&[0x0B])),
        OpusErrorKind::InvalidPacket
    );
    assert_eq!(
        kind(dec.packet_duration(&[0x0B, 0x30])),
        OpusErrorKind::InvalidPacket,
        "48 frames of 20 ms exceed 120 ms"
    );
}

#[test]
fn empty_packets_are_rejected() {
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");
    let mut out = vec![0i16; 5760];
    assert_eq!(kind(dec.decode(&[], &mut out)), OpusErrorKind::BadArg);
    assert_eq!(
        kind(dec.decode_fec(&[], &mut out, 960)),
        OpusErrorKind::BadArg
    );
}
