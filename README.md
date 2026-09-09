# rust-spout2

Rust bindings for Spout2 on Windows.

## DirectX 11 texture sender

`DirectXSender` sends BGRA8/RGBA8 UNORM D3D11 textures using GPU copies. It does
not create an OpenGL context or read pixels back to the CPU. The original
`Spout`/SpoutLibrary API remains available.

```rust,no_run
use rust_spout2::DirectXSender;
use std::{ffi::{c_void, CString}, ptr};

// Obtain these live COM pointers from your renderer (for example, with
// windows::core::Interface::as_raw). Null placeholders are rejected.
let device: *mut c_void = ptr::null_mut();
let texture: *mut c_void = ptr::null_mut();
let name = CString::new("My video output").unwrap();
unsafe {
    if let Some(mut sender) = DirectXSender::new(device, &name) {
        assert!(sender.send_texture(texture));
        sender.release_sender();
    }
}
```

The sender retains a COM reference to the device and releases its registration
on drop. Keep it on its creating thread. Enable D3D11 multithread protection
when sharing the immediate context with other threads. Submitted textures must
belong to that device, have one mip and array slice, and be single-sampled.
Resolution and BGRA/RGBA format changes are handled automatically. The caller
must own any keyed mutex protecting the source during submission. A busy
receiver may cause Spout to skip a frame. Names must contain 1–255 ANSI bytes;
use ASCII for portable names.

## Notes

- This crate provides direct bindings to SpoutLibrary and builds Spout2 from source during build.
- Windows-only behavior is expected.
- Source resolution order:
    1. `SPOUT2_DIR` environment variable (must point to existing Spout2 sources)
    2. Local `./Spout2` directory
    3. Optional auto-fetch when `RUST_SPOUT2_ALLOW_FETCH=1` (clones tag `2.007h`)

### Build setup

Install Visual Studio C++ Build Tools, Windows SDK, CMake, and LLVM/libclang
22.1.1 (the version tested with autocxx 0.26 and used in CI).
Place CMake on `PATH` and, if libclang is not found automatically, set
`LIBCLANG_PATH` to the LLVM `bin` directory. Distribute the generated runtime
DLLs alongside your executable when using SpoutLibrary, and retain
`THIRD-PARTY-NOTICES.txt` when redistributing Spout code or binaries.

```powershell
# Recommended: use pre-fetched sources
$env:SPOUT2_DIR = "C:/path/to/Spout2"

# Optional: allow build.rs to fetch sources when missing
$env:RUST_SPOUT2_ALLOW_FETCH = "1"
```

## Example

```rust
use rust_spout2::Spout;

fn main() {
    let mut spout = Spout::new().expect("failed to get Spout handle");
    let version = spout.as_pin_mut().GetSpoutVersion();
    println!("Spout version: {version}");
}
```
