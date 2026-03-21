# Changelog

## 0.1.1 - 2026-03-22

- Simplified API surface to direct SpoutLibrary bindings.
- Added a safe RAII wrapper `Spout` that calls `Release` on drop.
- Added explicit Windows-only compile guard.
- Improved `build.rs` source resolution (`SPOUT2_DIR`, optional fetch toggle).
- Stabilized runtime DLL copy destination path computation.
- Added Windows CI workflow running `cargo check`.
- Reduced generated-code warning noise in local checks.
