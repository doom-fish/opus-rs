# Coverage

What this crate wraps from libopus 1.5.2, and what is intentionally deferred.
The goal of `0.2.0` is the **single-stream encoder/decoder** path needed for
real-time WebRTC audio.

## Encoder (`OpusEncoder`)

| Capability | API | CTL |
| --- | --- | --- |
| Create / destroy | `Encoder::new` / `Drop` | `opus_encoder_create` / `_destroy` |
| Encode `i16` PCM | `Encoder::encode` | `opus_encode` |
| Bitrate (bps) | `set_bitrate` / `bitrate` | `OPUS_SET_BITRATE` / `_GET_`; a value of zero or less is rejected rather than read as auto (-1000) or max (-1) |
| Bitrate mode (auto/max/bps) | `set_bitrate_mode` | `OPUS_SET_BITRATE` |
| Complexity | `set_complexity` / `complexity` | `OPUS_SET_COMPLEXITY` / `_GET_` |
| VBR | `set_vbr` / `vbr` | `OPUS_SET_VBR` / `_GET_` |
| Constrained VBR | `set_vbr_constraint` / `vbr_constraint` | `OPUS_SET_VBR_CONSTRAINT` / `_GET_` |
| In-band FEC | `set_inband_fec` / `inband_fec` | `OPUS_SET_INBAND_FEC` / `_GET_` |
| Packet-loss % | `set_packet_loss_perc` / `packet_loss_perc` | `OPUS_SET_PACKET_LOSS_PERC` / `_GET_` |
| DTX | `set_dtx` / `dtx` | `OPUS_SET_DTX` / `_GET_` |
| Max bandwidth | `set_max_bandwidth` / `max_bandwidth` | `OPUS_SET_MAX_BANDWIDTH` / `_GET_` |
| Bandwidth | `set_bandwidth` | `OPUS_SET_BANDWIDTH` |
| Application | `set_application` / `application` | `OPUS_SET_APPLICATION` / `_GET_` |
| Signal hint | `set_signal` / `signal` | `OPUS_SET_SIGNAL` / `_GET_` |
| Lookahead (delay) | `lookahead` | `OPUS_GET_LOOKAHEAD` |
| Sample rate | `sample_rate` | `OPUS_GET_SAMPLE_RATE` |
| Reset | `reset_state` | `OPUS_RESET_STATE` |

## Decoder (`OpusDecoder`)

| Capability | API | CTL |
| --- | --- | --- |
| Create / destroy | `Decoder::new` / `Drop` | `opus_decoder_create` / `_destroy` |
| Decode to `i16` PCM | `Decoder::decode` | `opus_decode` |
| Packet-loss concealment (explicit frame size) | `Decoder::decode_lost` | `opus_decode` (null packet) |
| In-band FEC recovery (explicit frame size) | `Decoder::decode_fec` | `opus_decode` (`decode_fec = 1`) |
| Packet duration | `Decoder::packet_duration` | `opus_decoder_get_nb_samples` |
| FEC used last decode | `get_in_band_fec` | tracked client-side |
| Gain | `set_gain` / `gain` | `OPUS_SET_GAIN` / `OPUS_GET_GAIN` |
| Bandwidth | `bandwidth` | `OPUS_GET_BANDWIDTH` |
| Last packet duration | `last_packet_duration` | `OPUS_GET_LAST_PACKET_DURATION` |
| Sample rate | `sample_rate` | `OPUS_GET_SAMPLE_RATE` |
| Reset | `reset_state` | `OPUS_RESET_STATE` |

## Library

| Capability | API |
| --- | --- |
| Version string | `version()` (`opus_get_version_string`) |
| Error strings | `OpusError` (`opus_strerror`) |
| Raw FFI declarations | `opus::sys` |

## Deferred (not yet wrapped)

These libopus features are not exposed in `0.2.0`. None are required for the
single-stream real-time audio use case.

- **Multistream** encoder/decoder (`opus_multistream_*`).
- **Projection** ambisonics API (`opus_projection_*`).
- **Repacketizer** (`opus_repacketizer_*`) and packet inspection helpers other
  than the duration (`opus_packet_get_bandwidth`, `opus_packet_parse`, ...).
- **Surround** channel-mapping helpers.
- **DRED / deep-PLC** (`ENABLE_DEEP_PLC`) and **OSCE** (`ENABLE_OSCE`) — the
  vendored tree is stripped of `dnn/` and these are compiled out.
- **Custom modes**: `CUSTOM_MODES` is left undefined, as in upstream's default
  build, so the non-Opus custom-mode code paths are compiled out.
- **Float PCM** entry points (`opus_encode_float` / `opus_decode_float`); only
  the `i16` paths are wrapped. (The float *analysis* sources are still compiled
  because `DISABLE_FLOAT_API` is not defined.)
- SIMD intrinsic code paths (ARM NEON, x86 SSE/AVX) — disabled in favour of the
  portable fixed-point C, so one build serves every supported Apple target.
- **Non-Apple targets** — `build.rs` stops with an error; it only knows how to
  drive clang with Apple target triples and SDKs.
- Less common encoder CTLs: prediction-disabled, phase-inversion-disabled,
  expert frame-duration, LSB depth, force-channels.

## Build facts

- Vendored libopus: **1.5.2**, ~2.5 MB after stripping; all 252 vendored files
  are byte-identical to upstream `opus-1.5.2.tar.gz` (including
  `package_version`).
- `build.rs`: 449 lines, compiles 132 C translation units for the clang
  `--target` derived from Cargo's `TARGET`, with the deployment target, SDK
  and `CC`/`AR`/`CFLAGS` taken from the environment.
- Fixed-point build: `-DFIXED_POINT=1 -DOPUS_BUILD=1 -DENABLE_HARDENING=1`
  (upstream's default run-time checks: a broken internal invariant aborts
  instead of corrupting memory), no intrinsics, no
  `cc`/`bindgen`/`pkg-config`/`cmake`/`autoconf`.
- Runtime dependency: `libc` only.
