use std::path::{Path, PathBuf};

/// Official Spout2 source repository (Windows).
const SPOUT_DIR: &str = "Spout2";
const SPOUT_TAG: &str = "2.007h";
const SPOUT_DIR_ENV: &str = "SPOUT2_DIR";
const SPOUT_ALLOW_FETCH_ENV: &str = "RUST_SPOUT2_ALLOW_FETCH";

fn main() {
    let spout_source_dir = resolve_spout_source_dir();
    let (spout_build_dir, lib_dir) = build_spout(&spout_source_dir);
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
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-env-changed={}", SPOUT_DIR_ENV);
    println!("cargo:rerun-if-env-changed={}", SPOUT_ALLOW_FETCH_ENV);

    println!("cargo:rustc-link-lib=SpoutLibrary");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
}

fn copy_runtime_dlls(spout_build_dir: &Path) {
    let target_profile_dir = cargo_target_profile_dir();
    if let Err(e) = std::fs::create_dir_all(&target_profile_dir) {
        println!(
            "cargo:warning=Failed to create runtime output directory {}: {}",
            target_profile_dir.display(),
            e
        );
        return;
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

fn resolve_spout_source_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(SPOUT_DIR_ENV) {
        let path = PathBuf::from(dir);
        if path.exists() {
            return path;
        }
        panic!(
            "{} is set but path does not exist: {}",
            SPOUT_DIR_ENV,
            path.display()
        );
    }

    let local = PathBuf::from(SPOUT_DIR);
    if local.exists() {
        return local;
    }

    let allow_fetch = std::env::var(SPOUT_ALLOW_FETCH_ENV).as_deref() == Ok("1")
        || is_packaged_verification_context();

    if allow_fetch {
        if is_packaged_verification_context() {
            return fetch_spout2_into(out_dir().join("spout2-src"));
        }

        return fetch_spout2_into(local);
    }

    panic!(
        "Spout2 sources not found. Place sources at './{}' or set {} to an existing path. Optional: set {}=1 to allow automatic git clone.",
        SPOUT_DIR,
        SPOUT_DIR_ENV,
        SPOUT_ALLOW_FETCH_ENV
    );
}

fn fetch_spout2_into(dest: PathBuf) -> PathBuf {
    if dest.exists() {
        return dest;
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).expect("failed to create Spout2 checkout parent directory");
    }

    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            SPOUT_TAG,
            "https://github.com/leadedge/Spout2.git",
        ])
        .arg(&dest)
        .status()
        .expect("failed to execute git clone for Spout2");

    if status.success() {
        return dest;
    }

    panic!("Unable to clone Spout2 sources. Ensure git is installed and network access is available.");
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR env var is missing"))
}

fn is_packaged_verification_context() -> bool {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(_) => return false,
    };

    let normalized = manifest_dir.replace('\\', "/");
    normalized.contains("/target/package/")
}

fn build_spout(spout_source_dir: &Path) -> (PathBuf, PathBuf) {
    let dst = cmake::Config::new(spout_source_dir)
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

fn cargo_target_profile_dir() -> PathBuf {
    let profile = std::env::var("PROFILE").expect("PROFILE env var is missing");
    let target_triple = std::env::var("TARGET").expect("TARGET env var is missing");

    let target_dir = if let Ok(explicit_target_dir) = std::env::var("CARGO_TARGET_DIR") {
        PathBuf::from(explicit_target_dir)
    } else {
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var is missing"))
            .join("target")
    };

    target_dir.join(target_triple).join(profile)
}
