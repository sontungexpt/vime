# Vietnamese Fcitx5 Adapter

Independent Fcitx5 adapter project. Vietnamese processing remains in the
`../engine` Rust library.

The native C++ addon in `native/vime.cpp` implements
`fcitx::InputMethodEngine`; the Rust crate provides the C ABI backend. The
Wayland text-input frontend remains Fcitx5's responsibility.

Build the Rust backend:

```sh
cargo test
cargo build --release
```

Build and stage the complete addon with Fcitx5 development headers installed:

```sh
cmake -S native -B build
cmake --build build
cmake --install build --prefix "$PWD/stage"
```
