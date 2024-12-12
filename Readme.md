# Blinki starter for doing some christmas magic with rust on ESP32-C6

## Getting started
- Install IDE (vscode, rustrover or whatever else you like)
- Rust toolchain
  - [rust installation](https://dev.to/francescoxx/rust-installation-hello-world-1omk)
    - (when using rustrover the rust installation can be done directly in the ide without running extra commands)
  - [prerequisites](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-macos-setup.html#step-1-install-prerequisites)
- (optional) In case you want to generate a default template in the future, install: 
  - [cargo generate](https://github.com/cargo-generate/cargo-generate)
- Build the project
```sh
cargo build --release
```
- Flash the project to the esp connected via USB
```sh
cargo run --release
```

## Trouble shooting
- If you get an error like 
```
error: linker `ldproxy` not found
  |
  = note: No such file or directory (os error 2)

```
Then you just have to [install ldproxy](https://docs.esp-rs.org/book/troubleshooting/std.html#missing-ldproxy)
```sh
cargo install ldproxy

```

### Thanks for getting me started
- [yt video - shanemmattner](https://www.youtube.com/watch?v=vUSHaogHs1s&ab_channel=ShaneMattner)
- [article - Rajesh Pachaikani](https://medium.com/@rajeshpachaikani/connect-esp32-to-wifi-with-rust-7d12532f539b)