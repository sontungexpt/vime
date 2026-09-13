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
# development: consume the local checkout instead of cloning from GitHub
cmake -B build -DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE=/abs/path/vime/engine
cmake --build build        # imports vime-engine, builds its FFI crate, links libvime
cmake --install build --prefix "$HOME/.local"
```

Without the `FETCHCONTENT_SOURCE_DIR_VIME_ENGINE` override, CMake clones
`https://github.com/sontungexpt/vime-engine.git` into `build/_deps/` on first
configure.