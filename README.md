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

- **nvim** depends on the `vime-engine` crate via git. A gitignored
  `.cargo/config.toml` `[patch]` resolves it from the local checkout during
  development; release builds (without that file) resolve from GitHub.
- **fcitx5** pulls the repository via CMake `FetchContent`. When the sibling
  `../../engine` checkout is present it is used automatically; otherwise
  (release/standalone) it clones from GitHub.

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
cmake -B build              # auto-uses ../../engine when present; else clones from GitHub
cmake --build build         # builds vime-engine's FFI crate and links libvime
cmake --install build --prefix "$HOME/.local"
```

### Neovim

```sh
cd adapters/nvim && cargo check   # git dep; dev patch in .cargo/config.toml, GitHub in release
```