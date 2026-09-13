# Vietnamese Fcitx5 Adapter

Native Fcitx5 adapter plugin for Vime. Vietnamese engine logic lives in the
`vime-engine` companion repo at `../../engine` (`core` crate), exposed
via the C ABI in `../../engine/ffi`.

The native C++ addon implements `fcitx::InputMethodEngine`; the `ffi` crate
provides the C ABI backend (`libvime.so`).

## Build

Build the Rust FFI crate:

```sh
cargo build --release --manifest-path ../../engine/ffi/Cargo.toml
```

Build and install the Fcitx5 addon:

```sh
cmake -B build
cmake --build build
cmake --install build --prefix "$HOME/.local"
```
