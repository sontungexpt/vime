# Vietnamese Fcitx5 Adapter

Native Fcitx5 adapter plugin for Vime. Vietnamese engine logic lives in the
standalone `vime-engine` repository (github.com/sontungexpt/vime-engine), which
is pulled in via CMake `FetchContent`; the `ffi` crate (`libvime.so`) provides
the C ABI backend (`vime_engine.h`).

The native C++ addon implements `fcitx::InputMethodEngine` and links the
Rust-compiled `libvime.so` through the C ABI.

## Build

Build and install the Fcitx5 addon:

```sh
cmake -B build              # auto-detects the sibling ../../engine checkout in dev
cmake --build build         # builds its FFI crate and links libvime
cmake --install build --prefix "$HOME/.local"
```

`vime-engine` is resolved automatically: in a development layout (the engine
checkout next to this repo) CMake builds against the in-tree copy; otherwise it
clones `https://github.com/sontungexpt/vime-engine.git` into `build/_deps/` on
first configure — the release/standalone path. Force a specific location with
`-DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE=/path/vime/engine`.