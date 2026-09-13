# Vietnamese Fcitx5 Adapter

Native Fcitx5 adapter plugin for Vime. Core Vietnamese engine logic is in `../../engine`, exposed via C ABI in `../../ffi`.

The native C++ addon in `fcitx5.cpp` implements `fcitx::InputMethodEngine`; the `ffi` crate provides the C ABI backend.

## Build

Build the Rust FFI crate:

```sh
cargo build --release --manifest-path ../../ffi/Cargo.toml
```

Build and install the Fcitx5 addon:

```sh
cmake -B build
cmake --build build
cmake --install build --prefix "$HOME/.local"
```
