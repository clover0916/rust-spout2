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
    let bin_dir = spout_build_dir.join("bin");
    let target_profile_dirs = cargo_target_profile_dirs();

    if target_profile_dirs.is_empty() {
        println!(
            "cargo:warning=Could not determine runtime output directory from OUT_DIR={}.",
            out_dir().display()
        );
        return;
    }

    for dir in &target_profile_dirs {
        if let Err(e) = std::fs::create_dir_all(dir) {
            println!(
                "cargo:warning=Failed to create runtime output directory {}: {}",
                dir.display(),
                e
            );
        }
    }

    for dll_name in ["Spout.dll", "SpoutLibrary.dll"] {
        let src = bin_dir.join(dll_name);
        if !src.exists() {
            continue;
        }

        for target_profile_dir in &target_profile_dirs {
            let dst = target_profile_dir.join(dll_name);
            if let Err(e) = std::fs::copy(&src, &dst) {
                println!(
                    "cargo:warning=Failed to copy {} to {}: {}",
                    dll_name,
                    dst.display(),
                    e
                );
            }
        }
    }
}

fn resolve_spout_source_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(SPOUT_DIR_ENV) {
        let path = PathBuf::from(dir);
        if is_valid_spout_source_dir(&path) {
            return path;
        }
        panic!(
            "{} is set but path is not a valid Spout2 source directory (missing CMakeLists.txt): {}",
            SPOUT_DIR_ENV,
            path.display()
        );
    }

    let local = PathBuf::from(SPOUT_DIR);
    if is_valid_spout_source_dir(&local) {
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

fn is_valid_spout_source_dir(path: &Path) -> bool {
    path.join("CMakeLists.txt").is_file()
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

fn cargo_target_profile_dirs() -> Vec<PathBuf> {
    let mut result = Vec::new();
    let out = out_dir();

    // OUT_DIR typically resolves to:
    // <target>/<profile>/build/rust-spout2-*/out
    // or <target>/<triple>/<profile>/build/rust-spout2-*/out
    // Copy to both profile roots to support runners that start from either location.
    let mut current = out.as_path();
    while let Some(parent) = current.parent() {
        if current.file_name().and_then(|v| v.to_str()) == Some("build") {
            if let Some(profile_dir) = current.parent() {
                push_unique_path(&mut result, profile_dir.to_path_buf());
                if let Some(target_root) = profile_dir.parent() {
                    let profile_name = profile_dir
                        .file_name()
                        .map(|v| v.to_os_string())
                        .unwrap_or_default();
                    push_unique_path(&mut result, target_root.join(profile_name));
                }
            }
            break;
        }
        current = parent;
    }

    result
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|p| p == &path) {
        paths.push(path);
    }
}
