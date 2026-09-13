# Vime Vietnamese Input Method

Vime is organized into two repositories that live in one project for easy
management:

```
vime/                      # adapters repo (this one)
├── vime-engine/           # companion git repo — engine + C ABI
│   ├── engine/            #   pure Rust Vietnamese input-method core
│   └── ffi/               #   C ABI boundary (vime.h, libvime.so / libvime.a)
├── adapters/
│   └── fcitx5/            # native C++ Fcitx5 adapter plugin
└── nvim/                  # Neovim integration
```

`vime-engine/` is its own git repository (in `.gitignore` here); adapters
consume it via the `libvime.so` C ABI and never depend on the Rust toolchain
or engine internals.

## Building & Testing

### Engine + FFI (in `vime-engine/`)

```sh
cd vime-engine
cargo test --workspace
cargo build --release       # produces libvime.so / libvime.a
```

### Fcitx5 adapter

```sh
cd adapters/fcitx5
cmake -B build
cmake --build build          # builds ../vime-engine/ffi then links libvime
cmake --install build --prefix "$HOME/.local"
```

### Neovim

```sh
cd nvim && cargo check
```