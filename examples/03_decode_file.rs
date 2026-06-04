//! Decode length-prefixed Opus packets from stdin (as produced by
//! `02_encode_file`) back into 48 kHz mono `s16le` PCM on stdout.
//!
//! ```text
//! cat audio.opus.framed | cargo run --example 03_decode_file > audio.out.s16le
//! ```

#![allow(clippy::significant_drop_tightening)]

use opus::{Channels, Decoder, SampleRate};
use std::io::{self, Read, Write};

const FRAME_SAMPLES: usize = 960; // 20 ms @ 48 kHz mono

fn main() -> io::Result<()> {
    let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono).expect("create decoder");

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut framed = Vec::new();
    stdin.read_to_end(&mut framed)?;

    let mut pcm = vec![0i16; FRAME_SAMPLES];
    let mut pos = 0;
    while pos + 2 <= framed.len() {
        let len = u16::from_be_bytes([framed[pos], framed[pos + 1]]) as usize;
        pos += 2;
        if pos + len > framed.len() {
            break;
        }
        let packet = &framed[pos..pos + len];
        pos += len;

        let samples = dec.decode(packet, &mut pcm, false).expect("decode");
        for &s in &pcm[..samples] {
            stdout.write_all(&s.to_le_bytes())?;
        }
    }
    stdout.flush()
}
