# Changelog

All notable changes to `opus-rs` are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - Unreleased

### Security

- libopus is built with `CUSTOM_MODES` undefined, as upstream's default build
  is. `-DCUSTOM_MODES=0` defined the macro, and libopus tests it with
  `#ifdef`, so the non-default custom-mode code (23 exported functions such as
  `opus_custom_encoder_create`) was compiled in.
- `ENABLE_HARDENING` is defined, as in upstream's default build: internal
  assertions and encoder/decoder state validation abort instead of letting a
  broken invariant corrupt memory.

### Fixed

- Packet-loss concealment and FEC recovery produce exactly the requested
  duration. The whole output buffer was used as the frame size, so a
  5760-sample buffer gave 120 ms of concealment per lost 20 ms packet, and
  buffer lengths that were not a multiple of 2.5 ms failed.
- `build.rs` passes a clang `--target` triple with the deployment target and
  the matching SDK for Cargo's target (macOS, iOS, tvOS, watchOS and visionOS
  devices and simulators, Mac Catalyst) instead of only `-arch`, honours
  `CC`, `AR` and `CFLAGS`, and stops with a clear error on unsupported
  targets instead of panicking or building macOS objects for iOS.
- The `license` field is `(MIT OR Apache-2.0) AND BSD-3-Clause`, covering the
  vendored libopus.
- The documentation no longer claims the crate compiles on every
  architecture.

### Changed

- **Breaking:** `Decoder::decode(packet, output)` no longer takes a `fec`
  flag; use `Decoder::decode_fec` for in-band FEC recovery.
- **Breaking:** `Decoder::decode_lost(output, frame_size)` takes the duration
  to conceal in samples per channel, a multiple of 2.5 ms up to 120 ms.
- `Decoder::decode` and `Decoder::decode_fec` reject an empty packet with
  `BadArg`.
- `rust-version` is 1.82.

### Added

- `Decoder::decode_fec(packet, output, frame_size)`.
- `Decoder::packet_duration(packet)` (samples per channel of a packet).
- `vendor/libopus/package_version`, upstream's version file; the vendored tree
  was verified byte-identical to `opus-1.5.2.tar.gz`, and `build.rs` checks
  the recorded version.
- A `.gitignore` for build output.
- Tests for random and corrupted packets and for PLC/FEC frame sizing.

## [0.1.0] - 2026-06-04

### Added

- Initial release: safe Rust bindings over a vendored, statically-linked
  **libopus 1.5.2** (fixed-point build).
- `build.rs` compiles the vendored C sources by invoking `clang` directly via
  `std::process::Command` — no `cc`, `bindgen`, `pkg-config`, `cmake` or
  `autoconf`.
- `Encoder`: create/encode plus CTL setters and getters for bitrate (incl.
  `Bitrate` mode), complexity, VBR, constrained VBR, in-band FEC, packet-loss
  percentage, DTX, max bandwidth, bandwidth, application, signal, lookahead,
  sample rate and state reset.
- `Decoder`: decode, packet-loss concealment (`decode_lost`), gain set/get,
  bandwidth, last packet duration, sample rate and state reset.
- Strongly-typed `SampleRate`, `Channels`, `Application`, `Bandwidth`,
  `Signal` and `Bitrate` enums.
- `OpusError` / `OpusErrorKind` error type mapping every libopus error code.
- Raw FFI surface exposed under the `sys` module.
- Examples: `01_version`, `02_encode_file`, `03_decode_file`.
- Integration tests for round-trip encode/decode, frame sizes, FEC/PLC and CTL
  settings.

[0.2.0]: https://github.com/doom-fish/opus-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/doom-fish/opus-rs/releases/tag/v0.1.0
