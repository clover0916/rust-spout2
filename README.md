# rust-spout2

Rust bindings for Spout2 on Windows.

## Notes

- This crate provides direct bindings to SpoutLibrary and builds Spout2 from source during build.
- Windows-only behavior is expected.
- Source resolution order:
    1. `SPOUT2_DIR` environment variable (must point to existing Spout2 sources)
    2. Local `./Spout2` directory
    3. Optional auto-fetch when `RUST_SPOUT2_ALLOW_FETCH=1` (clones tag `2.007h`)

### Build setup

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
