// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Build script for `y4-scudo-sys`.
//!
//! Compiles the LLVM scudo standalone allocator (Apache-2.0 + LLVM
//! exception, sourced from `third_party/scudo-source/compiler-rt/lib/
//! scudo/standalone/`) into a static library and links it into the
//! crate.  The submodule is sparse-checked-out to that subdirectory to
//! avoid pulling the rest of the LLVM tree.
//!
//! This script targets *hosted Linux* builds — the seL4 / `no_std` build
//! requires platform shims (mmap, pthread, etc.) that scudo expects
//! and that `kernel/` will provide later.  For Phase B step 3 the
//! hosted backend is what the unit tests exercise.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scudo_dir = manifest.join("../third_party/scudo/standalone");

    println!("cargo:rerun-if-changed={}", scudo_dir.display());
    println!("cargo:rerun-if-changed=../third_party/scudo/PIN.toml");
    println!("cargo:rerun-if-changed=build.rs");

    if !scudo_dir.join("CMakeLists.txt").exists() {
        // Pinned source not yet materialised — emit a clear message
        // and let downstream see the linker error.  We do NOT clone
        // here (build-time network is bad practice).
        println!(
            "cargo:warning=scudo source missing at {}.  Run `just scudo-fetch` to materialise the pin from third_party/scudo/PIN.toml.",
            scudo_dir.display()
        );
        return;
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-nostdinc++")
        .define("SCUDO_DEBUG", Some("0"))
        // Prefix every malloc-family export with `scudo_` so we don't
        // clash with libc's malloc / free / aligned_alloc.
        .define("SCUDO_PREFIX_NAME", Some("scudo_"))
        .include(&scudo_dir)
        .include(scudo_dir.join("include"));

    // Common platform-agnostic sources.
    for f in [
        "checksum.cpp",
        "common.cpp",
        "crc32_hw.cpp",
        "flags.cpp",
        "flags_parser.cpp",
        "release.cpp",
        "report.cpp",
        "string_utils.cpp",
        "timing.cpp",
        "wrappers_c.cpp",
    ] {
        build.file(scudo_dir.join(f));
    }

    // Linux-specific platform sources.  Y4 hosted builds run on Linux;
    // a Fuchsia or kernel-mode build would swap these out.
    if cfg!(target_os = "linux") {
        for f in [
            "linux.cpp",
            "mem_map.cpp",
            "mem_map_linux.cpp",
            "condition_variable_linux.cpp",
            "report_linux.cpp",
        ] {
            build.file(scudo_dir.join(f));
        }
    }

    build.compile("scudo_standalone");

    // Link order: pthread (scudo uses it for arena init).
    println!("cargo:rustc-link-lib=pthread");
}
