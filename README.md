# Vime Vietnamese Input Method

Vime is organized into modular layers:

- `engine/`: standalone pure Rust Vietnamese input-method core engine
- `ffi/`: C ABI boundary exposing public C headers (`vime.h`) and types/conversions
- `adapters/`: native platform and IME framework adapters
  - `adapters/fcitx5/`: native C++ Fcitx5 adapter plugin

```text
vime/
├── ffi/                   # [C ABI Boundary] Tầng tiếp xúc với C/C++ Frontend
│   ├── include/
│   │   └── vime.h         # Header file C công khai cho C/C++ Frontend
│   └── src/
│       ├── lib.rs         # C-FFI entry points (vime_create, vime_process_key,...)
│       ├── types.rs       # C-compatible structs/enums (VimeOutput, VimeKeyEvent,...)
│       ├── convert.rs     # Chuyển đổi giữa C-Types <-> Rust Domain Types
│       └── logging.rs     # Bridge chuyển log từ Rust `log` crate sang C Callback
│
├── engine/                # [Pure Rust Engine] Core xử lý dấu, quy tắc tiếng Việt
│   ├── src/
│   │   ├── lib.rs
│   │   ├── engine.rs      # Engine điều phối chính (State Machine)
│   │   ├── composition/   # Quản lý chuỗi đang gõ (Preedit buffer)
│   │   ├── rule_engine/   # Luật gõ Telex, VNI, VIQR,...
│   │   ├── phonology/     # Cấu trúc âm tiết tiếng Việt, nguyên âm, phụ âm
│   │   └── renderer/      # Render và đặt dấu chính tả
│   └── Cargo.toml
│
└── adapters/              # Các adapter tĩnh bằng Rust/C++ cho từng OS/IME
    └── fcitx5/            # Fcitx5 C++ plugin wrapper
```

## Crates & Architecture

- `engine`: standalone Vietnamese input-method engine (`Engine`, `Buffer`, `Parser`, Telex/VNI rule engines, `decode_vowel`/`encode_vowel` codec, syllable normalization and rendering).
- `ffi`: C ABI boundary crate providing C declarations and types (`vime_create`, `vime_process_key`, `vime_reset`, `vime_set_input_method`, `vime_destroy`).
- `adapters/fcitx5`: native C++ Fcitx5 plugin implementing `fcitx::InputMethodEngine`.

## Building & Testing

### Rust Engine & FFI

```sh
cargo test --all
cargo build --release
```

### Fcitx5 Adapter

```sh
cd adapters/fcitx5
cmake -B build
cmake --build build
cmake --install build --prefix "$HOME/.local"
```
