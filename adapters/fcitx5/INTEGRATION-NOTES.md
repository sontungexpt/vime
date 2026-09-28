# fcitx5 integration: how the three layers fit, and what to build next

Analysis of the engine ↔ FFI ↔ fcitx5 chain, cross-checked against
[`fcitx5-lotus`](https://github.com/xmthanh/lotus) as a reference implementation.

The original gap analysis is kept in Part 6 below, unchanged. Everything above
it is new: the integration traced end to end, the cursor question answered
definitively, and a staged plan. **Nothing here is implemented.**

---

## Part 1 — the integration, end to end

### Three layers, and who owns what

| Layer | Repo | Language | Owns |
|---|---|---|---|
| Engine | `engine/core` (nested repo `vime-engine`) | Rust | parsing, cursor, rendering |
| FFI | `engine/ffi` | Rust → C ABI | handle, caches, text ownership |
| Adapter | `adapters/fcitx5` | C++ | fcitx types, panel, commit |

`fcitx5-lotus` has the same shape but swaps Rust for Go behind a C shim, so its
ABI decisions are comparable but not identical.

### The per-keystroke path, concretely

```
fcitx  →  VimeEngine::keyEvent            (adapters/fcitx5/src/engine.cpp:45)
        →  VimeState::keyEvent            (src/state.cpp:32)
            ├─ toVimeKeyEvent(key)        (include/converter.h:50)  keysym → VimeKeyEvent
            ├─ vime_process_key(h, ev)    (ffi/src/lib.rs:156)      engine runs
            │    └─ returns VimeOutput { action, commit }
            ├─ if action == FORWARD → return (key passes to the app)
            ├─ VimeState::apply(output)    (src/state.cpp:127)
            │    ├─ COMMIT  → output.commit → ic->commitString()
            │    │             + clear the preedit
            │    └─ CHANGED / CURSOR_MOVED → showPreedit()
            └─ event.filterAndAccept()
```

Two things are worth naming precisely.

**The engine runs before any fcitx decision.** `vime_process_key` is called
unconditionally and the adapter inspects the resulting action afterwards. That
is the same contract lotus uses (`lotus-state.cpp:16-19`): no early return
between "engine ran" and "panel is now correct".

**Forwarding is a return, not a call.** Our adapter returns early on
`VIME_ACTION_FORWARD` and otherwise calls `filterAndAccept()`. Lotus simply
never calls `filterAndAccept()` and lets fcitx forward. Equivalent behaviour,
but it splits our "did anything change" decision across two sites — the action
check *and* the accept call.

### Who owns the text

Ours is **borrowed and cached**; lotus's is **transferred**.

```c
const char *vime_parsed(VimeEngineHandle *engine);   // ours: handle-owned
```

The FFI renders lazily on first read after a state change and caches until the
next one (`ffi/src/types.rs:117-126`), so a frontend that only reacts to
`VIME_ACTION_COMMIT` never pays for a render. The pointer stays valid until the
next state-changing call or `vime_destroy`; callers must not free it.

Lotus instead does `EnginePullPreedit` → `UniqueCPtr<char>`, transferring an
allocation per keystroke. Ours is measurably cheaper, at the cost of a lifetime
rule we can only *document*, not enforce.

---

## Part 2 — the cursor question

> Does the preedit have a cursor, and can we change it?

**Yes on both counts.** The engine tracks it, the ABI reports that it moved, the
adapter repaints — and never tells fcitx where it is. One missing call, and one
missing ABI field.

### fcitx5 has a first-class cursor on the preedit

From `/usr/include/Fcitx5/Core/fcitx/text.h:34-37`:

```cpp
/// Get cursor by byte.
int cursor() const;
/// Set cursor by byte.
void setCursor(int pos = -1);
```

`fcitx::Text` — the type `setClientPreedit` takes — carries a **byte** offset,
not a character index. The distinction matters: every Vietnamese vowel is 2–3
UTF-8 bytes, so a 6-character syllable is 8+ bytes, and a character index passed
to `setCursor` would land mid-character and mis-render.

Lotus sets it on every render (`lotus-state.cpp:217`):

```cpp
text.setCursor(static_cast<int>(text.textLength()));
```

Our adapter never calls `setCursor`. `showPreedit()` (`state.cpp:101`) builds a
fresh `fcitx::Text` and hands it over, so the cursor keeps its default.

### The engine already tracks the position

This is the part that makes the fix cheap. The engine is not missing work:

- `Composition` holds two cursors, `raw_cursor` and `parsed_cursor`
  (`core/src/composition/mod.rs:33,36`), because the raw keystroke buffer and
  the rendered syllable have different lengths.
- `can_move_left` / `can_move_right` gate navigation (`:112`, `:121`).
- `move_left` / `move_right` move both, each clamped independently (`:130`).
- `Cursor::get()` returns the 0-based position (`cursor.rs:26`).
- `VIME_ACTION_CURSOR_MOVED` is in the ABI (`vime_engine.h:33`) and already
  handled in `apply()`.

So arrow keys move a real, correct cursor, the engine reports it, the adapter
repaints — and the caret is invisible, because nothing ever tells fcitx where it
is. Today `CursorMoved` costs a repaint to achieve nothing visible.

### The blocker is one missing field in the C ABI

`vime_parsed` returns a bare `const char *`. **There is nowhere in the ABI to
report "the caret is at byte 4."** Grepping the header for `cursor` finds only
the `VIME_ACTION_CURSOR_MOVED` enum value and one doc mention — no position.

So the fix has two halves:

1. **Engine/FFI** — expose the parsed cursor position, in **bytes**, alongside
   the rendered text. This is an ABI change.
2. **Adapter** — call `setCursor(byte_offset)` on the `fcitx::Text` before
   `setClientPreedit`.

A byte offset is the right unit, because that is what `Text` wants and what the
FFI already has in hand. Convert once, at the boundary, rather than making
every frontend guess at UTF-8 arithmetic.

---

## Part 3 — the original notes: what lotus does at the fcitx5 boundary

### The shape of a keystroke

`LotusState::keyEvent` runs the engine first, then *unconditionally* reconciles
the input panel — `ic_->inputPanel().reset()`, rebuild the `Text`, then
`ic_->updatePreedit()` and `ic_->updateUserInterface(InputPanel)`. There is no
early return between "engine ran" and "panel is now correct".

Forwarding is implicit: the handler simply does not call
`keyEvent.filterAndAccept()`, so fcitx passes the key to the application. Our
adapter instead checks the action and returns — equivalent, but it means our
"did anything change" decision is spread across two places.

### Capability flags gate the preedit call

```cpp
if (ic_->capabilityFlags().test(CapabilityFlag::Preedit))
    ic_->inputPanel().setClientPreedit(text);
else
    ic_->inputPanel().setPreedit(text);
```

Two different calls depending on what the frontend supports.

### The caret is always placed

```cpp
text.setCursor(static_cast<int>(text.textLength()));
```

Set on every render. This is the single most important thing they do that we do
not (see gap 2).

### Deactivate commits rather than discards

`LotusEngine::deactivate` commits the buffer when in preedit mode, and
distinguishes `InputContextFocusOut` from `InputContextReset`. `LotusEngine::reset`
additionally bails out early if the state has no history, so a focus event with
an empty buffer does not clear anything.

### Modes, not one behaviour

`LotusMode` is `Off / Smooth / SuperSmooth / Uinput / SurroundingText / Preedit
/ Emoji / Minecraft`. Preedit is one mode among several; the others rewrite the
application's existing text instead of using a preedit.

### App-awareness

`getProgramName(ic)` feeds `setAppRule(appName, mode)`, so the mode is
remembered per application. A `lotus-monitor` watches process changes.

### Autofill and ghost-text detection

`isAutofillCertain(surrounding)` inspects the application's surrounding text to
decide whether extra backspaces are needed. The comments name the bugs it
works around explicitly — a "toôi" duplication in Chromium search bars, and
distinguishing browser autofill from AI ghost text.

### The ABI shape differs from ours

| | lotus | ours |
|---|---|---|
| text access | `EnginePullPreedit` → `C.CString` | `vime_parsed` → `*const c_char` |
| ownership | caller frees (`UniqueCPtr<char>`) | handle owns, valid until next call |
| on every key | pull both preedit and commit | only when the action says so |

Their "pull" model transfers an allocation per keystroke. Ours borrows and
caches, which is why ours is measurably cheaper — but it means a caller that
holds a pointer across a call gets a dangling one, and ours can only document
that rule rather than enforce it.

---

## Part 4 — gaps in our adapter, most important first

### 1. Deactivate throws away a composed word — data loss

```cpp
void VimeEngine::deactivate(...) { state->reset(); }   // clears, discards
```

If the input method deactivates with a word half-typed, the text is silently
dropped. Lotus commits the buffer here. This is a bug, not a missing feature.

### 2. The caret is tracked but never shown

The engine has a cursor, `can_move_left`/`can_move_right`, and a dedicated
`VIME_ACTION_CURSOR_MOVED`. The adapter handles that action by calling
`showPreedit()` — the same code path as a text change, which never calls
`setCursor`. So arrow keys move a caret that is invisible, and
`CursorMoved` costs a repaint to achieve nothing visible.

The engine work is done and paid for; the adapter is the missing half.

### 3. No capability check before `setClientPreedit`

We always call `setClientPreedit`. On a frontend that lacks
`CapabilityFlag::Preedit` the call is a no-op and the user sees nothing. Lotus
falls back to `setPreedit` instead.

### 4. No `inputPanel().reset()` or `updateUserInterface(InputPanel)`

We call `setClientPreedit` + `updatePreedit`, but never reset the panel and
never notify the input-panel component. Harmless while we show no candidates;
will matter the moment we do.

### 5. Deactivate and reset are treated identically

Both call `reset()`. Lotus separates focus-out from reset, and skips the clear
when there is no history — so an innocuous focus event cannot wipe state.

### 6. We never look at the application's text

Lotus reads `surroundingText()` to learn the real caret position and to detect
autofill. We assume the caret is where we left it. This is fine for the preedit
model and is the main reason we cannot adopt lotus's faster in-place replacement
modes later without extra plumbing.

### 7. Modified keys are inserted as text — FIXED

**Fixed.** The guard is now in `VimeState::keyEvent` (`src/state.cpp`):

```cpp
if (key.hasModifier()) {
    return;   // let fcitx forward it to the app
}
```

The keysym of Ctrl+A is still `a`, so before this every shortcut reached the
engine and was inserted. Lotus has the same guard at `lotus-state.cpp:249`.

`test/modifier-key.cpp` is the regression test. Against the pre-fix build it
reproduces the bug exactly:

```
Ctrl+A  accepted=1 preedit='A'
  FAIL  Ctrl+A does not insert text
truowngf preedit='Ctruowngf'     <- the stray 'C' left behind by Ctrl+C
  FAIL  plain 'truowngf' still parses to 'trường'
```

That surfaced a second symptom worth naming: **stray shortcut letters
accumulate in the preedit and corrupt the next word.** Any buffer half-typed
before you pressed Ctrl+C came back as `Ctruowngf`. The "Ctrl makes everything
uppercase" report is this, compounded by the preedit being invisible until
commit (gap 3).

Not wired into CMake; the test carries its build command in a comment.

### 8. Ctrl+Backspace does nothing useful

The guard forwards it, which is correct but not useful — it would be nice as
"delete the whole syllable". That needs engine support for a range delete:
`remove` takes a character index, not a range. Left as a product decision.

### 9. No per-application behaviour, no mode switcher

Both are product features rather than correctness issues.

---

## Part 5 — suggested order of work

**Correctness first**

1. Fix deactivate to commit, not discard. The commit text now arrives on
   `VimeOutput::commit`, but a reset is not a commit and `vime_reset` returns
   `bool` with no text, so this needs a decision: have the adapter read
   `vime_parsed` and commit that, or give the engine an explicit
   "commit-on-deactivate"? The second keeps the state knowledge on the
   engine side, which is where it has ended up for every other path.
2. Honour `CapabilityFlag::Preedit`, with `setPreedit` as the fallback.
3. Split focus-out from reset, and skip the clear when the buffer is empty.

**Finish what the engine already does**

4. Plumb the caret through to fcitx. This needs the cursor position in the
   C ABI — currently `vime_parsed` returns a bare string, so there is nowhere
   to report "caret is at 3". Either add a `vime_parsed_cursor`, or have
   `vime_parsed` fill a caller-supplied struct. **This is an ABI decision and
   should be made before more frontends exist**, not after.
5. Add `inputPanel().reset()` and `updateUserInterface(InputPanel)` to the
   render path, so candidates behave when we add them.

**Then, only if wanted**

6. Per-application mode rules, once there is more than one mode.
7. Surrounding-text awareness, which is the prerequisite for the in-place
   replacement modes that make typing feel fast in browsers.

**Not planning to do**

- The backspace-counting, sleep-and-retry machinery for Chromium/Electron. It
  exists to work around commit-in-place races in the non-preedit modes. Our
  preedit mode does not send our own keystrokes to the application, so the
  problem does not arise. Worth revisiting only if we adopt those modes.
- `isAutofillCertain`. Same reason: it guards against text we did not type
  appearing in the application's buffer.

---

## Part 6 — plan, staged

### Stage 0 — correctness, no ABI change

Small, self-contained, and each one is a bug or a robustness gap.

| # | Change | File |
|---|---|---|
| 1 | `deactivate` commits instead of discarding | `src/engine.cpp` |
| 2 | Honour `CapabilityFlag::Preedit`, fall back to `setPreedit` | `src/state.cpp` |
| 3 | Distinguish focus-out from reset; skip the clear when empty | `src/engine.cpp` |

**On #1** there is a real design choice. The engine currently commits as a
*consequence* of processing a key, and the text arrives on the returned
`VimeOutput`. Deactivate has no key, so nothing triggers it. Two options:

- **(a) adapter-driven** — read `vime_parsed`, `commitString()` it, then reset.
  No ABI change. Downside: the adapter knows the buffer is non-empty because it
  checked, duplicating an engine invariant.
- **(b) engine-driven** — add an explicit `vime_commit()` that produces the text
  and clears the buffer, mirroring the existing commit path.

(b) is more consistent with how committing already works and keeps "what is the
engine's state" on the engine side. It costs one C entry point. **Recommend
(b).**

### Stage 1 — the cursor, end to end

The largest item, and the one that should land as a unit.

1. **Engine** — a `parsed_cursor_position()` returning the caret offset in
   bytes. Convert from the character index the engine holds, once, at the
   boundary. Decide and document the unit explicitly; `Text` wants bytes.
2. **FFI** — carry it. Two shapes:
   - a `vime_parsed_cursor(handle) -> int32_t` accessor, or
   - widen `VimeOutput` with a `caret` field.

   **Recommend the accessor.** `VimeOutput` is returned by value and is a small
   POD; adding a field changes its size and the ABI layout every frontend has
   already agreed on. A separate accessor follows the existing lazy-ownership
   model — produced when asked for, owned by the handle like `vime_parsed`.
3. **Adapter** — in `showPreedit()`, `text.setCursor(byte_offset)` before
   `setClientPreedit`. Do it unconditionally, including when the word is
   unchanged: that is exactly the `CursorMoved` case.
4. **Tests** — an FFI test asserting the offset tracks `move_left`/`move_right`,
   plus the byte-vs-character distinction with a multi-byte syllable (`nguyễn` is
   7 chars / 8 bytes). A character-index bug here renders as a mangled preedit,
   not a crash, so only a test catches it.

**This is an ABI decision and should be made before more frontends exist.** The
nvim adapter (`adapters/nvim`) would need the same field.

### Stage 2 — panel hygiene

Add `inputPanel().reset()` and
`updateUserInterface(UserInterfaceComponent::InputPanel)` to the render path, so
candidates behave when we add them. Do this alongside the first candidate work,
not before — until then it is untestable plumbing.

### Stage 3 — only if wanted

Per-application mode rules, once there is more than one mode.
Surrounding-text awareness, prerequisite for in-place replacement modes.

---

## Part 7 — verification notes

Findings above were read from the source, not assumed:

- `Text::setCursor` / `cursor()` — `/usr/include/Fcitx5/Core/fcitx/text.h:34-37`
- Lotus cursor + capability fallback — `fcitx5-lotus/src/lotus-state.cpp:205-224`
- Our render path — `adapters/fcitx5/src/state.cpp:101-167`
- Engine cursors — `engine/core/src/composition/mod.rs:33-130`,
  `engine/core/src/composition/cursor.rs:26`
- ABI has no position field — `engine/ffi/include/vime_engine.h:91-98,124-148`

The ABI shape (accessor vs. `VimeOutput` field) is a judgement call, not a
finding; it is flagged as a recommendation above.
