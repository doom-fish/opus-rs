# opus-rs

Safe Rust bindings to [libopus](https://opus-codec.org), the reference
implementation of the Opus audio codec, statically linked from a **vendored**
source tree.

The crate's library name is `opus`; the package name is `opus-rs`. It is a
workspace-internal crate (`publish = false`).

## Why this crate exists

The surrounding workspace is otherwise a pure-Rust / Apple-SDK binding family.
`opus-rs` is the **one explicit C-FFI exception**: no Apple framework ships an
Opus codec, yet standard WebRTC mandates Opus for audio interop between peers.
To keep latency-sensitive real-time audio working across every WebRTC peer, we
vendor and statically link libopus.

## How it is built — no `cc`, `bindgen`, `pkg-config`, `cmake` or `autoconf`

* **libopus 1.5.2** is vendored under `vendor/libopus/` (~2.5 MB, intrinsics,
  DNN and tests stripped). Every vendored file is byte-identical to the one in
  the upstream `opus-1.5.2.tar.gz`, and `vendor/libopus/package_version`
  records the version; `build.rs` refuses to build if it does not match.
* `build.rs` compiles it by invoking the C compiler (`clang` by default)
  directly through [`std::process::Command`]. There is **no** dependency on
  `cc`, `bindgen`, `pkg-config`, `cmake` or `autoconf`. The source allow-lists
  are hand-derived from upstream `*_sources.mk` files.
* The build is **fixed-point** (`-DFIXED_POINT=1`) with all SIMD intrinsics
  disabled. As in upstream's default build, custom modes are compiled out
  (`CUSTOM_MODES` undefined) and `ENABLE_HARDENING` is on, so a broken
  internal invariant aborts instead of corrupting memory.
* Only Apple targets are supported: macOS, iOS, tvOS, watchOS and visionOS
  (devices and simulators) and Mac Catalyst, on `aarch64`, `arm64e`,
  `arm64_32`, `armv7k`, `x86_64` and `x86_64h`. The clang `--target` triple
  follows Cargo's `TARGET`; the minimum OS comes from the platform's
  `*_DEPLOYMENT_TARGET` variable or, if unset, from
  `rustc --print deployment-target`; the SDK comes from `SDKROOT` (when it
  names the target's SDK) or `xcrun`. `CC`, `AR` and `CFLAGS` (including the
  `_<target>` and `TARGET_` forms) are honoured. Other targets fail with an
  error.
* A force-included header (`build/opus_buildtype_config.h`) supplies
  `PACKAGE_VERSION` so the version string resolves correctly without running
  `configure`.
* The compiled objects are archived with `ar` into a static `libopus.a`.
* The only runtime dependency is `libc`. Under `docs.rs` the C build is
  skipped.

## Quick start

```rust,no_run
use opus::{Encoder, Decoder, Application, Channels, SampleRate};

// 20 ms mono frame at 48 kHz = 960 samples.
let mut enc = Encoder::new(SampleRate::Hz48000, Channels::Mono, Application::Voip)?;
enc.set_bitrate(32_000)?;
enc.set_inband_fec(true)?;
enc.set_packet_loss_perc(10)?;

let pcm_in = vec![0i16; 960];
let mut packet = vec![0u8; 1500];
let n = enc.encode(&pcm_in, &mut packet)?;
let payload = &packet[..n];

let mut dec = Decoder::new(SampleRate::Hz48000, Channels::Mono)?;
let mut pcm_out = vec![0i16; 960];
let _m = dec.decode(payload, &mut pcm_out)?;
# Ok::<(), opus::OpusError>(())
```

For real-time use, always prefer **20 ms frames** between WebRTC peers.

For a lost packet, conceal exactly its duration with
`dec.decode_lost(&mut pcm_out, frame_size)`, or recover it from the next
packet's in-band FEC with `dec.decode_fec(&next, &mut pcm_out, frame_size)`.
`frame_size` is in samples per channel, a multiple of 2.5 ms up to 120 ms;
`dec.packet_duration(&next)?` gives a packet's duration.

## Examples

```bash
cargo run --example 01_version
cat audio.s16le | cargo run --example 02_encode_file > audio.framed
cat audio.framed | cargo run --example 03_decode_file > audio.out.s16le
```

(`02`/`03` operate on 48 kHz mono `s16le` PCM with 2-byte big-endian length
prefixes.)

## Raw FFI

The unsafe C declarations are exposed under [`opus::sys`] for callers that need
control beyond the safe wrappers.

## Coverage

See [`COVERAGE.md`](COVERAGE.md) for the exact list of libopus CTLs and APIs
that are wrapped versus deferred.

## License

The Rust binding code is licensed under either of

* MIT license ([LICENSE-MIT](LICENSE-MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

The vendored libopus sources under `vendor/libopus/` are licensed under the
BSD 3-Clause license; see [LICENSE-OPUS-BSD-3](LICENSE-OPUS-BSD-3) (copied
verbatim from the upstream `COPYING`). That notice must be retained in any
redistribution. The package's `license` field is therefore
`(MIT OR Apache-2.0) AND BSD-3-Clause`.
