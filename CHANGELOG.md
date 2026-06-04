# Changelog

All notable changes to `opus-rs` are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026

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

[0.1.0]: https://github.com/doom-fish/opus-rs/releases/tag/v0.1.0
