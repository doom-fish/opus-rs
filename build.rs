//! Build script for `opus-rs`.
//!
//! Statically compiles the vendored libopus 1.5.2 source tree into `libopus.a`
//! by invoking the C compiler (`clang` unless `CC` says otherwise) directly
//! through [`std::process::Command`]. No `cc`, `bindgen`, `pkg-config`,
//! `cmake` or `autoconf` are used — this is the one explicit C-FFI exception
//! in the workspace and it stays dependency-free.
//!
//! The build is **fixed-point** (`-DFIXED_POINT=1`) with all SIMD intrinsics
//! disabled, so the same portable C is compiled for every supported Apple
//! target. The clang target triple, deployment target and SDK follow Cargo's
//! `TARGET`; `CC`, `AR` and `CFLAGS` are honoured; other targets stop with an
//! error.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

/// Pinned libopus release. Must match `build/opus_buildtype_config.h`,
/// `CHANGELOG.md` and the vendored tree under `vendor/libopus/`.
const OPUS_VERSION: &str = "1.5.2";

/// CELT layer — portable base sources (no x86/arm intrinsics).
const CELT_SOURCES: &[&str] = &[
    "celt/bands.c",
    "celt/celt.c",
    "celt/celt_encoder.c",
    "celt/celt_decoder.c",
    "celt/cwrs.c",
    "celt/entcode.c",
    "celt/entdec.c",
    "celt/entenc.c",
    "celt/kiss_fft.c",
    "celt/laplace.c",
    "celt/mathops.c",
    "celt/mdct.c",
    "celt/modes.c",
    "celt/pitch.c",
    "celt/celt_lpc.c",
    "celt/quant_bands.c",
    "celt/rate.c",
    "celt/vq.c",
];

/// SILK layer — architecture-independent base sources.
const SILK_SOURCES: &[&str] = &[
    "silk/CNG.c",
    "silk/code_signs.c",
    "silk/init_decoder.c",
    "silk/decode_core.c",
    "silk/decode_frame.c",
    "silk/decode_parameters.c",
    "silk/decode_indices.c",
    "silk/decode_pulses.c",
    "silk/decoder_set_fs.c",
    "silk/dec_API.c",
    "silk/enc_API.c",
    "silk/encode_indices.c",
    "silk/encode_pulses.c",
    "silk/gain_quant.c",
    "silk/interpolate.c",
    "silk/LP_variable_cutoff.c",
    "silk/NLSF_decode.c",
    "silk/NSQ.c",
    "silk/NSQ_del_dec.c",
    "silk/PLC.c",
    "silk/shell_coder.c",
    "silk/tables_gain.c",
    "silk/tables_LTP.c",
    "silk/tables_NLSF_CB_NB_MB.c",
    "silk/tables_NLSF_CB_WB.c",
    "silk/tables_other.c",
    "silk/tables_pitch_lag.c",
    "silk/tables_pulses_per_block.c",
    "silk/VAD.c",
    "silk/control_audio_bandwidth.c",
    "silk/quant_LTP_gains.c",
    "silk/VQ_WMat_EC.c",
    "silk/HP_variable_cutoff.c",
    "silk/NLSF_encode.c",
    "silk/NLSF_VQ.c",
    "silk/NLSF_unpack.c",
    "silk/NLSF_del_dec_quant.c",
    "silk/process_NLSFs.c",
    "silk/stereo_LR_to_MS.c",
    "silk/stereo_MS_to_LR.c",
    "silk/check_control_input.c",
    "silk/control_SNR.c",
    "silk/init_encoder.c",
    "silk/control_codec.c",
    "silk/A2NLSF.c",
    "silk/ana_filt_bank_1.c",
    "silk/biquad_alt.c",
    "silk/bwexpander_32.c",
    "silk/bwexpander.c",
    "silk/debug.c",
    "silk/decode_pitch.c",
    "silk/inner_prod_aligned.c",
    "silk/lin2log.c",
    "silk/log2lin.c",
    "silk/LPC_analysis_filter.c",
    "silk/LPC_inv_pred_gain.c",
    "silk/table_LSF_cos.c",
    "silk/NLSF2A.c",
    "silk/NLSF_stabilize.c",
    "silk/NLSF_VQ_weights_laroia.c",
    "silk/pitch_est_tables.c",
    "silk/resampler.c",
    "silk/resampler_down2_3.c",
    "silk/resampler_down2.c",
    "silk/resampler_private_AR2.c",
    "silk/resampler_private_down_FIR.c",
    "silk/resampler_private_IIR_FIR.c",
    "silk/resampler_private_up2_HQ.c",
    "silk/resampler_rom.c",
    "silk/sigm_Q15.c",
    "silk/sort.c",
    "silk/sum_sqr_shift.c",
    "silk/stereo_decode_pred.c",
    "silk/stereo_encode_pred.c",
    "silk/stereo_find_predictor.c",
    "silk/stereo_quant_pred.c",
    "silk/LPC_fit.c",
];

/// SILK fixed-point layer (`SILK_SOURCES_FIXED`). Selected because we build
/// with `-DFIXED_POINT=1`; the `silk/float/*` counterpart is excluded.
const SILK_SOURCES_FIXED: &[&str] = &[
    "silk/fixed/LTP_analysis_filter_FIX.c",
    "silk/fixed/LTP_scale_ctrl_FIX.c",
    "silk/fixed/corrMatrix_FIX.c",
    "silk/fixed/encode_frame_FIX.c",
    "silk/fixed/find_LPC_FIX.c",
    "silk/fixed/find_LTP_FIX.c",
    "silk/fixed/find_pitch_lags_FIX.c",
    "silk/fixed/find_pred_coefs_FIX.c",
    "silk/fixed/noise_shape_analysis_FIX.c",
    "silk/fixed/process_gains_FIX.c",
    "silk/fixed/regularize_correlations_FIX.c",
    "silk/fixed/residual_energy16_FIX.c",
    "silk/fixed/residual_energy_FIX.c",
    "silk/fixed/warped_autocorrelation_FIX.c",
    "silk/fixed/apply_sine_window_FIX.c",
    "silk/fixed/autocorr_FIX.c",
    "silk/fixed/burg_modified_FIX.c",
    "silk/fixed/k2a_FIX.c",
    "silk/fixed/k2a_Q16_FIX.c",
    "silk/fixed/pitch_analysis_core_FIX.c",
    "silk/fixed/vector_ops_FIX.c",
    "silk/fixed/schur64_FIX.c",
    "silk/fixed/schur_FIX.c",
];

/// Top-level Opus glue (`OPUS_SOURCES`) plus `OPUS_SOURCES_FLOAT`. The float
/// sources (tonality analysis + MLP) are included because we keep the float
/// API enabled — we do NOT pass `-DDISABLE_FLOAT_API`.
const OPUS_SOURCES: &[&str] = &[
    "src/opus.c",
    "src/opus_decoder.c",
    "src/opus_encoder.c",
    "src/extensions.c",
    "src/opus_multistream.c",
    "src/opus_multistream_encoder.c",
    "src/opus_multistream_decoder.c",
    "src/repacketizer.c",
    "src/opus_projection_encoder.c",
    "src/opus_projection_decoder.c",
    "src/mapping_matrix.c",
    "src/analysis.c",
    "src/mlp.c",
    "src/mlp_data.c",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/libopus");
    println!("cargo:rerun-if-changed=build/opus_buildtype_config.h");
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let vendor = manifest_dir.join("vendor/libopus");
    let config_header = manifest_dir.join("build/opus_buildtype_config.h");

    // docs.rs has no need (and no permission) to compile C; skip the native
    // build there and rely on the FFI declarations alone for doc generation.
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let recorded = std::fs::read_to_string(vendor.join("package_version")).unwrap_or_default();
    if recorded.trim() != format!("PACKAGE_VERSION=\"{OPUS_VERSION}\"") {
        fail(&format!(
            "vendor/libopus/package_version does not record libopus {OPUS_VERSION}"
        ));
    }

    let target = env::var("TARGET").unwrap_or_default();
    let platform = apple_platform(&target);
    let clang_target = format!(
        "--target={}-apple-{}{}{}",
        platform.arch,
        platform.os,
        deployment_target(&target, platform.deployment_variable),
        platform.variant
    );
    let sysroot = sdk_root(platform.sdk);
    let (cc, cc_args) = tool(&target, "CC", "clang");
    let (ar, ar_args) = tool(&target, "AR", "ar");
    let cflags: Vec<String> = target_env(&target, "CFLAGS")
        .map(|flags| flags.split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default();

    let includes = [
        vendor.join("include"),
        vendor.join("celt"),
        vendor.join("silk"),
        vendor.join("silk/fixed"),
        vendor.join("src"),
        manifest_dir.join("build"),
    ];

    let mut objects = Vec::new();

    for rel in CELT_SOURCES
        .iter()
        .chain(SILK_SOURCES)
        .chain(SILK_SOURCES_FIXED)
        .chain(OPUS_SOURCES)
    {
        let src = vendor.join(rel);
        let obj = out_dir.join(format!("{}.o", rel.replace(['/', '.'], "_")));

        let mut cmd = Command::new(&cc);
        cmd.args(&cc_args)
            .arg("-c")
            .arg(&src)
            .arg("-o")
            .arg(&obj)
            .arg(&clang_target)
            .arg("-isysroot")
            .arg(&sysroot)
            // Portable fixed-point build, no intrinsics.
            .args([
                "-fPIC",
                "-O2",
                "-DNDEBUG",
                "-std=c99",
                "-DFIXED_POINT=1",
                "-DOPUS_BUILD=1",
                "-DHAVE_LRINTF=1",
                "-DVAR_ARRAYS=1",
                "-DENABLE_HARDENING=1",
            ])
            // Force-include our minimal config header (PACKAGE_VERSION etc.).
            .arg("-include")
            .arg(&config_header);

        for inc in &includes {
            cmd.arg("-I").arg(inc);
        }
        cmd.args(&cflags);

        let status = cmd
            .status()
            .unwrap_or_else(|e| fail(&format!("failed to run `{cc}` for {rel}: {e}")));
        if !status.success() {
            fail(&format!("`{cc}` failed compiling {rel}"));
        }

        objects.push(obj);
    }

    // Archive all objects into a static library with POSIX `ar`.
    let archive = out_dir.join("libopus.a");
    let _ = std::fs::remove_file(&archive);
    let status = Command::new(&ar)
        .args(&ar_args)
        .arg("rcs")
        .arg(&archive)
        .args(&objects)
        .status()
        .unwrap_or_else(|e| fail(&format!("failed to run `{ar}`: {e}")));
    if !status.success() {
        fail(&format!("`{ar}` failed building libopus.a"));
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=opus");
    println!("cargo:warning=opus-rs: built vendored libopus {OPUS_VERSION} (fixed-point, no intrinsics)");
}

fn fail(message: &str) -> ! {
    eprintln!("error: opus-rs: {message}");
    exit(1);
}

struct ApplePlatform {
    arch: &'static str,
    os: &'static str,
    variant: &'static str,
    sdk: &'static str,
    deployment_variable: &'static str,
}

fn unsupported(target: &str) -> ! {
    fail(&format!(
        "unsupported target `{target}`: the vendored libopus build supports Apple targets only \
         (macOS, iOS, tvOS, watchOS and visionOS devices and simulators, and Mac Catalyst) on \
         aarch64, arm64e, arm64_32, armv7k, x86_64 and x86_64h"
    ))
}

fn apple_platform(target: &str) -> ApplePlatform {
    if env::var("CARGO_CFG_TARGET_VENDOR").as_deref() != Ok("apple") {
        unsupported(target);
    }
    let arch = match target.split('-').next().unwrap_or_default() {
        "aarch64" => "arm64",
        "arm64e" => "arm64e",
        "arm64_32" => "arm64_32",
        "armv7k" => "armv7k",
        "x86_64" => "x86_64",
        "x86_64h" => "x86_64h",
        _ => unsupported(target),
    };
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();
    let simulator = abi == "sim" || (abi != "macabi" && arch.starts_with("x86_64"));
    let (os, variant, sdk, deployment_variable) = match (os.as_str(), abi.as_str(), simulator) {
        ("macos", _, _) => ("macosx", "", "macosx", "MACOSX_DEPLOYMENT_TARGET"),
        ("ios", "macabi", _) => ("ios", "-macabi", "macosx", "IPHONEOS_DEPLOYMENT_TARGET"),
        ("ios", _, true) => (
            "ios",
            "-simulator",
            "iphonesimulator",
            "IPHONEOS_DEPLOYMENT_TARGET",
        ),
        ("ios", _, false) => ("ios", "", "iphoneos", "IPHONEOS_DEPLOYMENT_TARGET"),
        ("tvos", _, true) => (
            "tvos",
            "-simulator",
            "appletvsimulator",
            "TVOS_DEPLOYMENT_TARGET",
        ),
        ("tvos", _, false) => ("tvos", "", "appletvos", "TVOS_DEPLOYMENT_TARGET"),
        ("watchos", _, true) => (
            "watchos",
            "-simulator",
            "watchsimulator",
            "WATCHOS_DEPLOYMENT_TARGET",
        ),
        ("watchos", _, false) => ("watchos", "", "watchos", "WATCHOS_DEPLOYMENT_TARGET"),
        ("visionos", _, true) => (
            "xros",
            "-simulator",
            "xrsimulator",
            "XROS_DEPLOYMENT_TARGET",
        ),
        ("visionos", _, false) => ("xros", "", "xros", "XROS_DEPLOYMENT_TARGET"),
        _ => unsupported(target),
    };
    ApplePlatform {
        arch,
        os,
        variant,
        sdk,
        deployment_variable,
    }
}

fn deployment_target(target: &str, variable: &str) -> String {
    println!("cargo:rerun-if-env-changed={variable}");
    if let Some(version) = env::var(variable)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
    {
        return version;
    }
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    Command::new(rustc)
        .args(["--print", "deployment-target", "--target", target])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .rsplit_once('=')
                .map(|(_, version)| version.trim().to_owned())
        })
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| {
            fail(&format!(
                "cannot determine the deployment target for `{target}`; set {variable}"
            ))
        })
}

fn sdk_root(sdk: &str) -> String {
    println!("cargo:rerun-if-env-changed=SDKROOT");
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    if let Ok(root) = env::var("SDKROOT") {
        let matches_target = Path::new(&root)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().starts_with(sdk));
        if matches_target {
            return root;
        }
    }
    Command::new("xcrun")
        .args(["--sdk", sdk, "--show-sdk-path"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| {
            fail(&format!(
                "cannot locate the {sdk} SDK with xcrun; install Xcode (or the Command Line \
                 Tools for macOS) or point SDKROOT at the SDK"
            ))
        })
}

fn target_env(target: &str, name: &str) -> Option<String> {
    let underscored = target.replace('-', "_");
    [
        format!("{name}_{target}"),
        format!("{name}_{underscored}"),
        format!("TARGET_{name}"),
        name.to_owned(),
    ]
    .into_iter()
    .find_map(|key| {
        println!("cargo:rerun-if-env-changed={key}");
        env::var(&key).ok().filter(|value| !value.trim().is_empty())
    })
}

fn tool(target: &str, name: &str, default: &str) -> (String, Vec<String>) {
    let value = target_env(target, name).unwrap_or_else(|| default.to_owned());
    let mut words = value.split_whitespace().map(str::to_owned);
    let program = words.next().unwrap_or_else(|| default.to_owned());
    (program, words.collect())
}
