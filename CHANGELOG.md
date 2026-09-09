# Changelog

## 0.1.4 - 2026-09-10

- Added `DirectXSender` for D3D11 BGRA/RGBA GPU texture sharing without OpenGL or CPU readback.
- Retain the supplied COM device, reject unsupported textures and device mismatches, and release sender registration on drop.
- Compile the SpoutDX bridge against the existing pinned Spout2 SDK; preserve the SpoutLibrary API.
- Update autocxx to 0.30 to support Float16 types in current Windows/libclang headers.

## 0.1.3 - 2026-03-22

- Fixed Spout2 source discovery to require `CMakeLists.txt` instead of only directory existence.
- Prevented packaged-registry builds from selecting an incomplete `Spout2/` directory.
- `SPOUT2_DIR` now validates source layout and emits a clearer error when invalid.

## 0.1.2 - 2026-03-22

- Fixed `build.rs` runtime DLL copy destination detection for dependency builds.
- Runtime DLLs (`Spout.dll`, `SpoutLibrary.dll`) are now copied using `OUT_DIR`-derived target paths instead of the crate source tree.
- Improved warnings with full destination paths when DLL copy fails.

## 0.1.1 - 2026-03-22

- Simplified API surface to direct SpoutLibrary bindings.
- Added a safe RAII wrapper `Spout` that calls `Release` on drop.
- Added explicit Windows-only compile guard.
- Improved `build.rs` source resolution (`SPOUT2_DIR`, optional fetch toggle).
- Stabilized runtime DLL copy destination path computation.
- Added Windows CI workflow running `cargo check`.
- Reduced generated-code warning noise in local checks.
