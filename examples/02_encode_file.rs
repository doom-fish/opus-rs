//! Encode raw 48 kHz mono `s16le` PCM from stdin into length-prefixed Opus
//! packets on stdout.
//!
//! Frame size is fixed at 20 ms (960 samples). Each output packet is written
//! as a 2-byte big-endian length followed by the packet bytes, which
//! `03_decode_file` reads back.
//!
//! ```text
//! cat audio.s16le | cargo run --example 02_encode_file > audio.opus.framed
//! ```

#![allow(clippy::significant_drop_tightening)]

use opus::{Application, Channels, Encoder, SampleRate};
use std::io::{self, Read, Write};

const FRAME_SAMPLES: usize = 960; // 20 ms @ 48 kHz mono

fn main() -> io::Result<()> {
    let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)
        .expect("create encoder");
    enc.set_bitrate(32_000).expect("set bitrate");
    enc.set_inband_fec(true).expect("set fec");
    enc.set_packet_loss_perc(10).expect("set loss perc");

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut raw = Vec::new();
    stdin.read_to_end(&mut raw)?;

    let mut packet = vec![0u8; 4000];
    for chunk in raw.chunks_exact(FRAME_SAMPLES * 2) {
        let pcm: Vec<i16> = chunk
            .chunks_exact(2)
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        let n = enc.encode(&pcm, &mut packet).expect("encode");
        let len = u16::try_from(n).expect("packet fits in u16");
        stdout.write_all(&len.to_be_bytes())?;
        stdout.write_all(&packet[..n])?;
    }
    stdout.flush()
}
