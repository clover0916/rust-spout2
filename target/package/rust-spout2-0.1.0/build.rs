use std::path::{Path, PathBuf};

/// The Spout2 fork that does not include precompiled `dll`s and `lib`s.
const SPOUT_DIR: &str = "Spout2-lean";
const SPOUT_TAG: &str = "2.007.011";

fn main() {
    ensure_spout_initted();
    let (spout_build_dir, lib_dir) = build_spout();
    copy_runtime_dlls(&spout_build_dir);

    let mut cxx_builder = autocxx_build::Builder::new(
        "src/lib.rs",
        &[spout_build_dir.join("include/SpoutLibrary")],
    )
    .build()
    .unwrap();
    cxx_builder
        .flag_if_supported("-std=c++14")
        .compile("spoutlib");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lib.rs");

    println!("cargo:rustc-link-lib=SpoutLibrary");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
}

fn copy_runtime_dlls(spout_build_dir: &Path) {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut target_profile_dir = PathBuf::from(out_dir);
    for _ in 0..3 {
        target_profile_dir.pop();
    }

    let bin_dir = spout_build_dir.join("bin");
    for dll_name in ["Spout.dll", "SpoutLibrary.dll"] {
        let src = bin_dir.join(dll_name);
        if !src.exists() {
            continue;
        }

        let dst = target_profile_dir.join(dll_name);
        if let Err(e) = std::fs::copy(&src, &dst) {
            println!("cargo:warning=Failed to copy {}: {}", dll_name, e);
        }
    }
}

fn ensure_spout_initted() {
    if !Path::new(SPOUT_DIR).exists() {
        let status = std::process::Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                SPOUT_TAG,
                "https://github.com/virtual-puppet-project/Spout2-lean.git",
                SPOUT_DIR,
            ])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Unable to clone Spout2-lean sources");
        }
    }
}

fn build_spout() -> (PathBuf, PathBuf) {
    let dst = cmake::Config::new(SPOUT_DIR)
        .define("SKIP_INSTALL_ALL", "OFF")
        .define("SKIP_INSTALL_HEADERS", "OFF")
        .define("SKIP_INSTALL_LIBRARIES", "OFF")
        .define("SPOUT_BUILD_CMT", "OFF")
        // The only one we want
        .define("SPOUT_BUILD_LIBRARY", "ON")
        .define("SPOUT_BUILD_SPOUTDX", "OFF")
        .define("SPOUT_BUILD_SPOUTDX_EXAMPLES", "OFF")
        .build();

    (dst.clone(), dst.join("lib"))
}
