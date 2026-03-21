# rust-spout2

Rust bindings for Spout2 on Windows.

## Notes

- This crate wraps SpoutLibrary and builds Spout2 from source during build.
- Windows-only behavior is expected.
- `build.rs` fetches `Spout2-lean` tag `2.007.011` if sources are not present.

## Example

```rust
use rust_spout2::RustySpout;

fn main() {
    let mut spout = RustySpout::new().expect("failed to init spout");
    let version = spout.get_spout_version().expect("failed to get version");
    println!("Spout version: {version}");
}
```
