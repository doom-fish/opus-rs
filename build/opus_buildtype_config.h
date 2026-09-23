/*
 * opus_buildtype_config.h — minimal handwritten build configuration for the
 * vendored libopus 1.5.2 fixed-point build used by opus-rs.
 *
 * This replaces the autotools/CMake generated `config.h`. It is force-included
 * into every libopus translation unit via clang's `-include` flag from
 * build.rs, so it does NOT rely on the `HAVE_CONFIG_H` machinery.
 *
 * We deliberately keep this tiny: the actual feature selection (FIXED_POINT,
 * OPUS_BUILD, VAR_ARRAYS, ENABLE_HARDENING, intrinsics-off; CUSTOM_MODES left
 * undefined) is driven by `-D` flags on the clang command line, not from
 * here. Only the two values that libopus genuinely reads from a config header
 * live here.
 */
#ifndef OPUS_BUILDTYPE_CONFIG_H
#define OPUS_BUILDTYPE_CONFIG_H

/* Reported by opus_get_version_string(); pinned to the vendored release. */
#define PACKAGE_VERSION "1.5.2"

/* We always have a C99 <stdint.h> on every target opus-rs supports. */
#define HAVE_STDINT_H 1

#endif /* OPUS_BUILDTYPE_CONFIG_H */
