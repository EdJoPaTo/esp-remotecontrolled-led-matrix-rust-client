# ESP WLAN LED Matrix Rust Client

Control an espPixelmatrix via Rust.
See [ESP WLAN LED Matrix](https://github.com/EdJoPaTo/esp-remotecontrolled-led-matrix).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies.esp-remotecontrolled-led-matrix-client]
git = "https://github.com/EdJoPaTo/esp-remotecontrolled-led-matrix-rust-client"
branch = "main"
```

You also might want to use `features`, check the [the Cargo.toml of this project](https://github.com/EdJoPaTo/esp-remotecontrolled-led-matrix-rust-client/blob/main/Cargo.toml) for existing features.

```toml
[dependencies.esp-remotecontrolled-led-matrix-client]
git = "https://github.com/EdJoPaTo/esp-remotecontrolled-led-matrix-rust-client"
branch = "main"
default-features = false
features = ["async-tokio", "embedded-graphics-impl"]
```
