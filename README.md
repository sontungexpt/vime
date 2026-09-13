# Vime Vietnamese Input Method

Vime is organized into two repositories that live in one project for easy
management:

```
vime/                      # adapters repo (this one)
├── engine/                # vime-engine git repo (github.com/sontungexpt/vime-engine)
│   ├── core/              #   pure Rust Vietnamese input-method core crate
│   └── ffi/               #   C ABI boundary (vime_engine.h, libvime.so / libvime.a)
├── adapters/
│   ├── fcitx5/            # native C++ Fcitx5 adapter plugin
│   └── nvim/              # Neovim integration
```

`engine/` is the standalone `vime-engine` repository (in `.gitignore` here),
kept in-tree only to work on everything at once. Adapters do **not** reference
it by folder path — they consume it as a dependency:

- **nvim** depends on the `vime-engine` crate via git (with a `[patch]`
  override to the local checkout during development).
- **fcitx5** pulls the repository via CMake `FetchContent` (a
  `FETCHCONTENT_SOURCE_DIR_VIME_ENGINE` override uses the local checkout).

## Building & Testing

### Engine + FFI (in `engine/`)

```sh
cd engine
cargo test --workspace
cargo build --release       # produces libvime.so / libvime.a
```

### Fcitx5 adapter

```sh
cd adapters/fcitx5
# development: build against the local checkout
cmake -B build -DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE="$HOME/.../vime/engine"
cmake --build build         # fetches or uses local vime-engine, the FFI, links libvime
cmake --install build --prefix "$HOME/.local"
```

### Neovim

```sh
cd adapters/nvim && cargo check   # resolves vime-engine from the local checkout via [patch]
```