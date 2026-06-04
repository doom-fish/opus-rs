//! Round-trip every CTL setter/getter the safe API exposes.

use opus::{Application, Bandwidth, Bitrate, Channels, Decoder, Encoder, SampleRate, Signal};

#[test]
fn encoder_ctl_setters_and_getters() {
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");

    enc.set_bitrate(48_000).expect("set bitrate");
    assert_eq!(enc.bitrate().expect("get bitrate"), Bitrate::BitsPerSecond(48_000));

    enc.set_bitrate_mode(Bitrate::Max).expect("set bitrate max");
    // OPUS_BITRATE_MAX resolves to a concrete per-frame maximum on read-back,
    // so we just assert it round-trips to *some* explicit bitrate.
    assert!(matches!(
        enc.bitrate().expect("get bitrate"),
        Bitrate::BitsPerSecond(_)
    ));

    enc.set_complexity(7).expect("set complexity");
    assert_eq!(enc.complexity().expect("get complexity"), 7);

    enc.set_vbr(false).expect("set vbr");
    assert!(!enc.vbr().expect("get vbr"));
    enc.set_vbr(true).expect("set vbr");
    assert!(enc.vbr().expect("get vbr"));

    enc.set_vbr_constraint(true).expect("set vbr constraint");
    assert!(enc.vbr_constraint().expect("get vbr constraint"));

    enc.set_inband_fec(true).expect("set fec");
    assert!(enc.inband_fec().expect("get fec"));

    enc.set_packet_loss_perc(15).expect("set loss");
    assert_eq!(enc.packet_loss_perc().expect("get loss"), 15);

    enc.set_dtx(true).expect("set dtx");
    assert!(enc.dtx().expect("get dtx"));

    enc.set_max_bandwidth(Bandwidth::Wideband).expect("set max bw");
    assert_eq!(enc.max_bandwidth().expect("get max bw"), Bandwidth::Wideband);

    enc.set_bandwidth(Bandwidth::Fullband).expect("set bw");

    enc.set_application(Application::Audio).expect("set app");
    assert_eq!(enc.application().expect("get app"), Application::Audio);

    enc.set_signal(Signal::Music).expect("set signal");
    assert_eq!(enc.signal().expect("get signal"), Signal::Music);

    assert!(enc.lookahead().expect("lookahead") > 0);
    assert_eq!(enc.sample_rate().expect("sample rate"), 48_000);

    enc.reset_state().expect("reset");
}

#[test]
fn decoder_ctl_setters_and_getters() {
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("decoder");

    dec.set_gain(0).expect("set gain");
    assert_eq!(dec.gain().expect("get gain"), 0);
    dec.set_gain(256).expect("set gain");
    assert_eq!(dec.gain().expect("get gain"), 256);

    assert_eq!(dec.sample_rate().expect("sample rate"), 48_000);

    // These query last-packet state; before decoding they should still return
    // a value (not error).
    let _ = dec.last_packet_duration().expect("last packet duration");
    assert!(!dec.get_in_band_fec(), "no decode yet => no fec");

    dec.reset_state().expect("reset");
}

#[test]
fn bad_arguments_are_errors() {
    // Complexity out of range must fail.
    let mut enc =
        Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip).expect("encoder");
    assert!(enc.set_complexity(99).is_err(), "complexity 99 rejected");
}
