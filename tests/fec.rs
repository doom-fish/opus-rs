//! Exercise in-band FEC and packet-loss concealment.

use opus::{Application, Channels, Decoder, Encoder, SampleRate};

const FRAME: usize = 960; // 20 ms @ 48 kHz mono

fn noisy_frame(seed: &mut u32) -> Vec<i16> {
    // Cheap LCG to make non-trivial audio so FEC has something to encode.
    (0..FRAME)
        .map(|_| {
            *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((*seed >> 16) as i16) / 4
        })
        .collect()
}

#[test]
fn fec_encode_and_decode_lost() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    enc.set_bitrate(24_000).expect("bitrate");
    enc.set_inband_fec(true).expect("fec on");
    enc.set_packet_loss_perc(30).expect("loss perc");
    assert!(enc.inband_fec().expect("get fec"), "FEC reports enabled");
    assert_eq!(enc.packet_loss_perc().expect("get loss"), 30);

    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    let mut seed = 0x1234_5678;
    let mut packet = vec![0u8; 4000];
    let mut out = vec![0i16; FRAME];

    // Encode frame A, then frame B (which carries FEC for A).
    let a = noisy_frame(&mut seed);
    let na = enc.encode(&a, &mut packet).expect("encode A");
    let pkt_a = packet[..na].to_vec();

    let b = noisy_frame(&mut seed);
    let nb = enc.encode(&b, &mut packet).expect("encode B");
    let pkt_b = packet[..nb].to_vec();

    // Normal decode of A.
    let m = dec.decode(&pkt_a, &mut out, false).expect("decode A");
    assert_eq!(m, FRAME);

    // Simulate A being lost: decode B with fec=true to recover A from B's FEC.
    let recovered = dec.decode(&pkt_b, &mut out, true).expect("decode B fec");
    assert_eq!(recovered, FRAME, "FEC decode yields a full frame");
}

#[test]
fn packet_loss_concealment() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    let mut seed = 0xABCD_1234;
    let mut packet = vec![0u8; 4000];
    let mut out = vec![0i16; FRAME];

    // Decode one real packet to warm the decoder.
    let pcm = noisy_frame(&mut seed);
    let n = enc.encode(&pcm, &mut packet).expect("encode");
    dec.decode(&packet[..n], &mut out, false).expect("decode");

    // Now conceal a lost packet.
    let concealed = dec.decode_lost(&mut out).expect("plc");
    assert_eq!(concealed, FRAME, "PLC produces a full frame");

    // The in-band-fec helper should reflect the last decode (non-FEC here).
    assert!(!dec.get_in_band_fec());
}
