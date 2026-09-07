# Vime Vietnamese Input Method

Vime is organized as three independent projects:

- `engine/`: standalone Vietnamese input-method core
- `fcitx5/`: Fcitx5 adapter and addon
- `nvim/`: Neovim integration

Each directory is an independent Cargo project with its own lockfile.

```text
                 ┌─────────────────────────────────────────────┐
                 │              FRONTENDS (independent crates) │
                 │       fcitx5/  ·  nvim/                     │
                 └──────────────────────┬──────────────────────┘
                                        │ Input
                                        ▼
    engine/src  ┌─────────────────────────────────────────────┐
                │  Engine   (engine.rs)  Input ─▶ Result      │
                │        owns  Config + Composition           │
                │                  │                          │
                │                  ▼                          │
                │  Composition (composition.rs)              │
                │   raw ASCII buffer + cursor, renormalizes  │
                │        │ KeyContext          │ staged chars │
                │        ▼                    ▼               │
                │  Interpreter (interpreter/) Processor (processor/)
                │   config · telex · vni      apply · parse · normalize
                │   key ─▶ Operation          render · Orthography
                │        │                     │              │
                │        ▼                     ▼              │
                │  Operation (operation.rs) ★ shared middle   │
                │  Character (character/) ◀───┘               │
                │      Vowel · codec                          │
                └─────────────────────────────────────────────┘
```

Layers only depend downward: `character` has no dependencies, `processor`
consumes the codec, `interpreter` produces [`Operation`]s, `composition`
drives the interpreter and applies processor rules, `engine` wires it all
together for a frontend.

### Module dependencies

| Module                    | Depends on                                            | Depended on by                    |
|---------------------------|-------------------------------------------------------|-----------------------------------|
| `character/` (vowel, codec) | —                                                    | operation · interpreter · processor |
| `operation.rs`            | `character::{Shape, Tone}`                            | interpreter · composition · processor |
| `interpreter/` (config, telex, vni) | `character::{decode_vowel, BaseVowel, Shape, Tone}` · `operation` | composition · engine           |
| `processor/` (normalize, syllable, render, rules, tone) | `character::{Vowel, Tone}`            | composition                      |
| `composition.rs`          | `interpreter` · `processor` · `operation`             | engine                           |
| `config.rs`               | `interpreter` · `processor::Orthography`              | engine                           |
| `state.rs`                | `composition`                                         | engine                           |
| `input.rs` / `result.rs`  | —                                                    | engine                           |
| `engine.rs`               | composition · config · interpreter · input · result · state | lib (public API)            |
| `lib.rs`                  | all of the above (re-exports the public surface)      | `fcitx5/` · `nvim/`               |

### One keystroke end to end

```text
 Key 'a' ──▶ Engine::input(Input::Character('a'))
              └─ Composition::insert(a, interpreter, orthography)
                   ├─ context = KeyContext { target: char left of cursor }
                   ├─ Operation from interpreter
                   │     Insert(a)      → push 'a'
                   │     Shape(Tilde…)  → push marker letter (w / d / base)
                   │     Tone(Acute…)   → push tone key (s f r x j)
                   │     RemoveTone     → push 'z'
                   ├─ canonicalize_into → merge duplicate horns, relocate tones
                   ├─ parse            → syllable grammar → Valid / not
                   │     Valid   → serialize + render_word → rendered
                   │     invalid → keep staged verbatim (backspace-safe)
                   └─ Result::Changed
                        │
                        ▼
 frontend ──▶ Engine::state()  State { composition, rendered }  → preedit
 Space/Enter ──▶ Result::Commit(String) → commit + clear buffer
```

## Crates

- `engine`: standalone Vietnamese input-method core — `Engine`, `Composition`,
  layout-independent `Interpreter`/`Processor` split, Telex + VNI configs,
  `decode_vowel`/`encode_vowel` codec, syllable normalize and render.
- `fcitx5`: C ABI backend plus native C++ Fcitx5 adapter.
- `nvim`: Lua entry point and optional Rust support library.

## Core usage

```rust
use vietnamese_engine::{Config, Engine, Input};

let mut engine = Engine::new(Config::default());
for character in "aas".chars() {
    engine.input(Input::Character(character));
}
assert_eq!(engine.rendered(), "ấ");
```

The core stores raw composition separately from rendered output. For example,
raw `aas` renders as normalized Vietnamese `ấ`. Frontends read `engine.state()`
to display or commit the preedit.

## Test without Fcitx5

```sh
(cd engine && cargo test)
printf 'aas ' | (cd engine && cargo run --example vietnamese-cli)
```

## Fcitx5 integration

Fcitx5 addons use a C++ ABI. The adapter in `fcitx5/native/vime.cpp` is installed as
`vime.so`, subclasses `fcitx::InputMethodEngine`, implements `keyEvent`,
`activate`, `reset`, and `listInputMethods`, and exports
`fcitx_addon_factory_instance` with `FCITX_ADDON_FACTORY`.

The adapter translates Fcitx5 `KeyEvent` values to core `KeyEvent` values and
maps `EngineAction` values to `InputContext::commitString`,
`InputPanel::setClientPreedit`, `InputContext::updatePreedit`, and related
Fcitx5 operations. The Wayland frontend remains Fcitx5's responsibility; the
core does not implement a Wayland protocol.
