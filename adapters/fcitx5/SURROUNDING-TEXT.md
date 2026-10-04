# Surrounding text, explained against our adapter

A ground-up explanation of fcitx5's *surrounding text*, written against
`adapters/fcitx5` and `engine/` as they exist today. Everything is worked
through with concrete traces: host text, engine buffer, preedit, and the exact
fcitx5 and FFI calls at each keystroke.

**Nothing in this note is implemented.** It is a design note.

---

## 0. The one idea, up front

Our engine only knows about characters the user typed *since the last commit*.
The moment we call `commitString`, the text leaves our engine and becomes the
application's. From then on the application owns it and we are blind to it.

Surrounding text is the single channel that lets us look back at the
application's text around the caret. That is all it is. Everything else —
in-place replacement, undoing a tone on an already-committed word, detecting a
truncated window — is built on top of that one read.

So the question this note answers is: **what does that read cost us, and what
does it buy?**

---

## 1. The cast

Five things, and it helps to name them because the words get mixed up.

| # | Thing | Lives in | What it is |
|---|---|---|---|
| 1 | **Client** | GTK, Chromium, Firefox | the app; owns the real text buffer |
| 2 | **Frontend** | fcitx5's `wayland` addon | translates the client's protocol into fcitx5 types; caches it |
| 3 | **fcitx5 core** | `libFcitx5Core` | owns `InputContext`, routes keys, holds preedit |
| 4 | **Our adapter** | `adapters/fcitx5` (`vime.so`) | C++ `InputMethodEngine`; decides commit vs preedit |
| 5 | **Our engine** | `engine/ffi` (`libvime.so`) → `engine/core` | Rust; parsing, cursor, rendering |

The call chain for one keystroke, as it is today:

```
client (GTK)
  └─ sends a key, and separately `surrounding_text`
      └─ wayland frontend stores it on the InputContext
          └─ fcitx5 routes the key to us
              └─ VimeEngine::keyEvent            adapters/fcitx5/src/engine.cpp:70
                  └─ VimeState::keyEvent         adapters/fcitx5/src/state.cpp:26
                      └─ vime_session_*           engine/ffi/src/lib.rs
                          └─ Session (Rust)       engine/core/src/session/session.rs
```

Surrounding text enters the chain at step 2 and is **sitting on the
`InputContext` the whole time**, waiting for us to read it. Nobody pushes it at
us; we go get it. That is why it is cheap — it is already in fcitx5's memory, no
IPC on our side.

### The nvlm adapter

`adapters/nvim` sits at the same layer and will need the same decisions, but
its host (Neovim) has different capabilities: no surrounding text concept at
all, and a preedit is drawn by the editor plugin. Treat the engine-side changes
as shared, and the host-side changes as separate work.

---

## 2. The three texts

This is the part that causes most confusion, so here it is with numbers before
any code.

There are **three** distinct pieces of text at any moment:

| Name | Owner | Visible to user? | Reversible? |
|---|---|---|---|
| **Host text** | the application | yes, permanently | no |
| **Preedit** | fcitx5 | yes, underlined, not yet real | n/a |
| **Engine buffer** | us | no | yes, we hold both forms |

A fourth thing exists that people call "text" and is not text at all: the
**raw keystrokes**. For `chaof` the engine holds:

```
raw keys   c h a o f      (5 chars, what the fingers did)
rendered   c h à o        (4 chars, what the user meant)
```

Note they have **different lengths**. This is the single most important fact
about Vietnamese input methods and it is why a character index into one is
meaningless in the other. Our core already models this with two cursors
(`raw_cursor` / `rendered_cursor`).

### Walkthrough: typing a word, preedit mode, no surrounding text

Host starts empty. Caret at char 0.

| Keystroke | Raw keys | Engine buffer (rendered) | Preedit | Host text |
|---|---|---|---|---|
| — | `` | `` | `` | `""` |
| `c` | `c` | `c` | `c` | `""` |
| `h` | `ch` | `ch` | `ch` | `""` |
| `a` | `cha` | `cha` | `cha` | `""` |
| `o` | `chao` | `chao` | `chao` | `""` |
| `f` | `chaof` | `chào` | `chào` | `""` |
| `space` | `` | `` | `` | `"chào"` |

The `f` row is the whole point of the engine: five keys in, four characters
out. The host only ever receives the final four, on the last row.

This is the model we implement today, and it works. It works *because we never
need to look backwards*: everything we need is in the engine buffer.

---

## 3. Why surrounding text is needed: the committed-word problem

Now the same user, a moment later.

Host text is `"chào"` — committed, out of our control. Caret at char 4.

The user wants `chao`. They know the tone is wrong, so they press `f` again,
because in Telex pressing the same tone key twice removes the tone.

But `f` is a *fresh keypress* into an *empty engine buffer*. Our engine has no
idea there is a `chào` sitting there. All it can do is insert `f` and produce
`f`. The user gets `chàof`. Broken.

To do the right thing the engine needs to answer one question before it can
process the key:

> Is there a word immediately before the caret that *we* produced, and what keys
> produced it?

The application's text gives us half the answer ("there is `chào` there"). The
other half — "we produced it, from `chaof`" — only we can know, because we are
the ones who committed it. **Neither source alone is sufficient.** That is the
architectural reason this feature needs a new engine-side data structure and not
just a new read call.

### Walkthrough: the tone-undo, with surrounding text

State before the keypress:

```
host text        "chào"          chars: c(0) h(1) à(2) o(3)      cursor = 4
our history      we committed "chào" from raw keys "chaof"
engine buffer    (empty)
```

User presses `f`.

| Step | Who | What happens | Host text |
|---|---|---|---|
| 1 | fcitx5 | `keyEvent` fires | `"chào"` |
| 2 | adapter | read surrounding text, hand to engine | `"chào"` |
| 3 | engine | look up history: does the text before the caret match a word we committed? | `"chào"` |
| 4 | engine | **match** → refill buffer with raw keys `chaof` | `"chào"` |
| 5 | engine | now apply `f` as a real keystroke → tone already present, same key → remove tone | `"chào"` |
| 6 | engine | report: delete 4 chars before caret, insert `chao` | `"chào"` |
| 7 | adapter | `deleteSurroundingText(-4, 4)` then insert | `"chao"` |

Rendered result: `"chao"`. Four characters, not five. The `f` was consumed as a
*modifier*, exactly as if the word had never been committed.

Step 4 is the whole trick. Instead of re-parsing `chào` from the application's
text — which is genuinely ambiguous, because `chào` could have come from `chào`,
`chao` + some other key, or a literal paste — we look the text up in our own
commit history and recover the original keystrokes *exactly*.

This matters more than it sounds. Consider `"hoà"`. Was it typed `hoa` then
toned? `hoa` then `hoaf`? Or did the user literally type `h`,`o`,`a` and then
select-all and paste `hoà`? Re-parsing cannot tell you. The history can,
because the history is keyed by text and only contains text we produced.

---

## 4. What fcitx5 actually gives us

Verified against fcitx5 master (`src/lib/fcitx/surroundingtext.h`,
`src/lib/fcitx/inputcontext.h`, `src/lib/fcitx-utils/capabilityflags.h`).

### Reading

```cpp
// On our VimeState, which already holds ic_.
const fcitx::SurroundingText &st = ic_->surroundingText();

st.isValid();          // may be false even when the capability is advertised
st.text();             // const std::string &  — UTF-8, a window, NOT the document
st.cursor();           // unsigned int, UCS4 characters
st.anchor();           // unsigned int, UCS4 characters
st.selectedText();     // std::string — non-empty iff anchor != cursor
```

Two things to internalise:

**It is a window, not the document.** On Wayland `text-input-v3` the client
decides how much to send, and many send a limited slice around the cursor. We
have no idea where the window starts in the document. Section 8 covers why that
is dangerous.

**`cursor() != anchor()` means a selection exists.** If the user has selected
text, the region between them is *their* text, not something we typed.

### Writing

```cpp
ic_->deleteSurroundingText(int offset, unsigned int size);   // negative offset = before the caret
ic_->commitString(std::string_view text);
ic_->commitStringWithCursor(std::string_view text, size_t cursor);
ic_->updatePreedit();
```

### Units — the thing that will bite you

From `inputcontext.h`, the interface the frontends implement:

```cpp
/**
 * @param offset offset of deletion start, in UCS4 char.
 * @param size length of the deletion in UCS4 char.
 */
virtual void deleteSurroundingTextImpl(int offset, unsigned int size) = 0;
```

So at the fcitx5 library boundary, **everything is in UCS4 characters**, not
bytes:

| Thing | Unit |
|---|---|
| `SurroundingText::cursor()` | characters |
| `SurroundingText::anchor()` | characters |
| `deleteSurroundingText(offset, size)` | characters |
| `commitStringWithCursor(text, cursor)` | characters |
| `fcitx::Text::setCursor(pos)` | **bytes** (see below) |

The Wayland protocol itself is in bytes; the frontend converts. And note the
last row: `fcitx::Text::cursor()` is documented as *"Get cursor by byte"*,
which is a different unit from everything above it. That asymmetry already bit
us once — see `INTEGRATION-NOTES.md` Part 2.

### Capability flags

Reported by the *client*, not by us. `fcitx-utils/capabilityflags.h`:

```cpp
enum class CapabilityFlag : uint64_t {
    Preedit              = 1 << 1,
    Password             = 1 << 3,
    SurroundingText      = 1 << 6,
    NoOnScreenKeyboard   = 1 << 15,
    GetIMInfoOnFocus     = 1 << 23,
    ...
};
```

Read with `ic_->capabilityFlags().test(CapabilityFlag::SurroundingText)`.

The flag means "this client is *able* to report surrounding text". It does not
mean it is currently reporting any, and it certainly does not mean it reports it
correctly. Section 8.

---

## 5. Learning that the text changed

There is no `updateSurroundingText` callback on `InputMethodEngineV2` — the one
that exists in `inputmethodengine.h` is `FCITXCORE_DEPRECATED`. The current
pattern is an event watcher that sets a flag, and all work deferred to the next
keypress.

From `fcitx5-unikey`, the closest existing Vietnamese engine:

```cpp
// In the engine constructor.
eventWatchers_.emplace_back(instance_->watchEvent(
    EventType::InputContextSurroundingTextUpdated,
    EventWatcherPhase::PostInputMethod, [this](fcitx::Event &event) {
        auto &icEvent = static_cast<fcitx::InputContextEvent &>(event);
        auto *ic = icEvent.inputContext();
        auto *state = ic->propertyFor(&factory_);
        state->mayRebuildFromSurroundingText_ = true;
    }));
```

Two details worth copying exactly:

**The watcher only sets a boolean.** It does not touch the host and does not
call into the engine. Doing real work during event dispatch races with whatever
the client is in the middle of.

**The work happens at the top of `keyEvent`** — see below. Reading surrounding
text is only ever worth doing when a key is actually arriving.

```cpp
void VimeEngine::keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &keyEvent) {
    auto *ic = keyEvent.inputContext();
    auto *state = ic->propertyFor(&stateFactory_);
    state->absorbFromSurroundingTextIfPending();   // <-- here, before anything else
    state->keyEvent(keyEvent);
}
```

---

## 6. The adapter, concretely

Below is the shape of the change, written against our existing `VimeState`. It
is a sketch of the design, not code to paste.

### State additions

```cpp
// include/state.h
class VimeState final : public fcitx::InputContextProperty {
public:
    void keyEvent(fcitx::KeyEvent &event);
    void reset();
    void apply(const VimeOutput &output);

private:
    /// Set by the event watcher; consumed by absorbFromSurroundingTextIfPending().
    bool mayAbsorb_{false};

    /// True when the client told us it is a password field.
    bool password_{false};

    /// True when the engine reported that the buffer came from host text.
    bool absorbed_{false};

    void absorbFromSurroundingTextIfPending();
    void pushSurroundingTextToEngine();
    void drainCommit();
    void showPreedit();

    VimeEngine *engine_{nullptr};
    fcitx::InputContext *ic_{nullptr};
    VimeSessionHandle *session_{nullptr};
};
```

### Arming the flag

```cpp
// src/engine.cpp — VimeEngine constructor
eventWatchers_.emplace_back(instance_->watchEvent(
    fcitx::EventType::InputContextSurroundingTextUpdated,
    fcitx::EventWatcherPhase::PostInputMethod,
    [this](fcitx::Event &event) {
        auto &icEvent = static_cast<fcitx::InputContextEvent &>(event);
        auto *state = icEvent.inputContext()->propertyFor(&stateFactory_);
        if (state) {
            state->mayAbsorb_ = true;
        }
    }));
```

```cpp
// src/engine.cpp — activate()
void VimeEngine::activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    auto *ic = event.inputContext();
    auto *state = ic->propertyFor(&stateFactory_);
    if (!state) {
        return;
    }
    if (ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
        state->mayAbsorb_ = true;
    }
}
```

```cpp
// src/engine.cpp — reset()
void VimeEngine::reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    auto *ic = event.inputContext();
    auto *state = ic->propertyFor(&stateFactory_);
    if (!state) {
        return;
    }
    state->reset();
    if (event.type() == fcitx::EventType::InputContextReset
        && ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
        state->mayAbsorb_ = true;
    }
}
```

### Reading it

```cpp
// src/state.cpp
void VimeState::pushSurroundingTextToEngine()
{
    const auto &st = ic_->surroundingText();

    if (!st.isValid()) {
        // Tell the engine there is nothing, so a stale value cannot linger.
        vime_session_set_surrounding(session_, "", 0, 0, 0);
        return;
    }

    // `text()` is a window into the application's buffer. We copy it, because
    // fcitx5's copy is only valid for the duration of this call and the engine
    // may hold on to it until the next keystroke.
    const std::string &text = st.text();
    vime_session_set_surrounding(
        session_,
        text.data(),
        static_cast<uint32_t>(text.size()),   // bytes in
        st.cursor(),                          // characters
        st.anchor());                         // characters
}
```

Note the units crossing the boundary: **bytes in** (it is a C string),
**characters for the offsets**. The engine walks the UTF-8 once to convert. That
is the same walk `render.rs::measure` already does, so it is not new machinery.

### The absorb, per keystroke

```cpp
void VimeState::absorbFromSurroundingTextIfPending()
{
    absorbed_ = false;

    if (!mayAbsorb_ || !session_) {
        return;
    }
    mayAbsorb_ = false;

    // A password field must never have its text read, transformed or echoed.
    if (ic_->capabilityFlags().test(fcitx::CapabilityFlag::Password)) {
        return;
    }

    if (!ic_->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText)) {
        return;
    }

    // A non-empty selection is the user's selection, not a word we typed.
    if (!ic_->surroundingText().selectedText().empty()) {
        return;
    }

    pushSurroundingTextToEngine();

    // The engine decides whether the text before the caret is a word we
    // committed. It replays the original keystrokes if so, and reports how many
    // characters it adopted.
    absorbed_ = vime_session_absorb_surrounding(session_) > 0;

    if (absorbed_) {
        // In "modify" mode the word leaves the host and becomes our preedit.
        vime_session_take_host_span(session_);
        drainCommit();   // performs deleteSurroundingText + commit
    }
}
```

Everything that decides *whether* is ours; everything that decides *what* is the
engine's. The engine is the only component that knows what it committed.

### Applying an output

```cpp
void VimeState::apply(const VimeOutput &output)
{
    drainCommit();

    if (output.needPreedit) {
        showPreedit();
    } else if (!ic_->inputPanel().clientPreedit().text().empty()) {
        ic_->inputPanel().setClientPreedit(fcitx::Text(""));
        ic_->updatePreedit();
    }
}

void VimeState::drainCommit()
{
    VimeCommit c{};
    if (vime_session_take_commit(session_, &c) == 0) {
        return;
    }

    if (c.deleteBeforeChars > 0) {
        ic_->deleteSurroundingText(-static_cast<int>(c.deleteBeforeChars),
                                   c.deleteBeforeChars);
    }
    if (c.deleteAfterChars > 0) {
        ic_->deleteSurroundingText(0, c.deleteAfterChars);
    }
    if (c.commitLenBytes > 0) {
        ic_->commitStringWithCursor(
            std::string_view(c.commitPtr, c.commitLenBytes),
            c.commitCursorChar);
    }
}
```

`delete` before `commit`, always. And **both or neither** — see section 8.

### The preedit, with the caret finally set

```cpp
void VimeState::showPreedit()
{
    VimeSnapshot snap{};
    vime_session_snapshot(session_, &snap);

    fcitx::Text text(std::string_view(snap.renderedPtr, snap.renderedLenBytes));
    text.setCursor(static_cast<int>(snap.renderedCursorBytes));  // bytes!

    if (ic_->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
        ic_->inputPanel().setClientPreedit(text);
    } else {
        ic_->inputPanel().setPreedit(text);
    }
    ic_->updatePreedit();
}
```

---

## 7. The engine, concretely

### The commit history

The data structure the whole feature rests on.

```rust
// engine/core/src/session/history.rs  (proposed)

/// One word we handed to the application, and the keys that produced it.
///
/// The rendered text is the key, not the value: on absorb we look up whatever
/// is currently in the host by its text, and recover the keys from that. A word
/// whose text we do not recognise is not ours, and is left alone.
#[derive(Debug, Clone)]
pub struct CommittedWord {
    /// Rendered text, as committed. `à` is one char, two bytes.
    pub rendered: String,
    /// The exact keystrokes, in order: `chaof`.
    pub raw: Vec<char>,
}

pub const HISTORY_CAPACITY: usize = 16;

#[derive(Debug, Default)]
pub struct CommitHistory {
    words: VecDeque<CommittedWord>,
}

impl CommitHistory {
    /// Records a word we just committed, so we can undo it later.
    pub fn record(&mut self, rendered: String, raw: Vec<char>) {
        if self.words.len() == HISTORY_CAPACITY {
            self.words.pop_front();
        }
        self.words.push_back(CommittedWord { rendered, raw });
    }

    /// The longest suffix of `host_before_cursor` that we committed.
    ///
    /// Longest-first, so `chào` prefers the 4-char entry over any 2-char entry
    /// that happens to match its tail.
    pub fn match_suffix(&self, host_before_cursor: &[char]) -> Option<&CommittedWord> {
        self.words.iter().rev().find(|w| {
            host_before_cursor.len() >= w.rendered.len()
                && host_before_cursor[host_before_cursor.len() - w.rendered.len()..]
                    == w.rendered.chars().collect::<Vec<_>>()[..]
        })
    }

    pub fn clear(&mut self) {
        self.words.clear();
    }
}
```

Why a fixed capacity of 16: it bounds memory at a few hundred bytes per
session, it needs no allocation after warm-up, and it covers any realistic
"the user just typed this". A `VecDeque` is used rather than a `Vec` so
eviction is O(1).

### The absorb

```rust
// engine/core/src/session/session.rs  (proposed, next to `raw()`)

impl Session<KM> {
    /// Adopts the word immediately before the host caret, if we produced it.
    ///
    /// Refills the raw keystroke buffer from commit history so that the next
    /// keystroke is processed exactly as if the word had never been committed.
    /// Returns the number of characters adopted (0 = nothing matched).
    pub fn absorb_surrounding(&mut self) -> usize {
        let Some(surrounding) = self.surrounding.as_ref() else {
            return 0;
        };
        let before: Vec<char> = surrounding.before_cursor().collect();
        let Some(word) = self.history.match_suffix(&before) else {
            return 0;
        };

        let count = word.rendered.chars().count();

        // Replay the original keys. `push` is the same path the original
        // keystrokes took, so the rebuilt state is bit-for-bit what it was —
        // including any mid-word cursor position.
        self.reset();
        for ch in &word.raw {
            self.composition.push_raw(*ch);
        }

        self.absorbed_from_host = count;
        count
    }
}
```

Replaying the keys is the reason this is correct rather than approximate. If we
instead tried to seed the parsed syllable directly from `chào`, we'd have to
invent a mapping that may not exist. Replaying is exact by construction.

`absorbed_from_host` is the record the commit path needs: it says "these `count`
characters currently in the host are duplicated in our buffer and must be
removed when we write back".

### Recording on commit

```rust
/// Called when the engine decides a word is finished.
fn record_commit(&mut self) {
    let rendered = String::from_iter(self.composition.rendered_chars());
    if rendered.is_empty() {
        return;
    }
    let raw = self.composition.raw().to_vec();
    self.history.record(rendered, raw);
}
```

### Turning that into a delete + insert

`VimeInsertResult` already carries the piece of arithmetic we need:

```rust
pub enum InsertOutcome {
    /// The buffer simply grew.
    Extended,
    /// The buffer changed shape; `first_changed` is where it happened.
    Transformed { first_changed: usize },
}
```

So on a keypress that transforms:

```rust
/// The host span this operation replaces, and the text to put there.
fn commit_span(&self, first_changed: usize) -> (usize, String) {
    let rendered: Vec<char> = self.composition.rendered_chars().collect();

    // Everything from `first_changed` onward is rewritten by the transform, so
    // that is the span the host must give back. For `chao` + `f` -> `chao`,
    // first_changed is 2, the delete is 2 characters, and we insert `ao`.
    let delete_from = first_changed.min(rendered.len());
    let rewritten = rendered[delete_from..].iter().collect::<String>();

    (delete_from, rewritten)
}
```

Worked example, with the history match already having refilled the buffer with
`chaof`:

```
raw keys before     chaof      (5 chars)
first_changed            2     -- the tone marker on 'a' is what changed
rendered before     chào      (4 chars)   <- what the host has right now
rendered after      chao      (4 chars)

delete before caret: characters [2..4]  ==  "ào"
insert:                          "ao"

host: "chào"  ->  delete 2 chars  ->  "ch"  ->  insert "ao"  ->  "chao"
```

Note what makes this cheap: we delete **2** characters, not the whole 4. The
shared prefix `"ch"` is left untouched. This is the "in-place replacement" that
makes typing feel fast, and it falls out of `first_changed` for free.

---

## 8. The guard cases

These are the reasons the feature is experimental everywhere it exists. Each one
is a way to corrupt a user's document, not a cosmetic issue.

### 8a. The truncated window

The document is `"tiếngViệt"`. The caret is at the end, char 9. Suppose the
frontend only gave us a 4-character window:

```
window text   "Việt"      cursor 4
character before the window:  'g'   <- a Vietnamese letter
```

We scan backwards from the caret looking for a word start. Every character is
Vietnamese, so the scan runs off the **front of the window** and stops at index
0. From the window alone, `"Việt"` looks like a complete word.

It is not. The real word is `"Việt"` as part of a longer run, and if we absorb
it we have only *part* of what is in the host.

This is exactly the guard in `fcitx5-unikey`, and it is the one piece of their
logic that must be copied verbatim:

```cpp
// fcitx5-unikey/src/unikey-im.cpp:264-270
// Check if surrounding is not in a bigger part of word.
if (start != text.begin()) {
    if (isVnChar(utf8::getLastChar(text.begin(), start))) {
        return;   // the window is truncated; refuse to guess
    }
}
```

Applied to our example:

```
scan from caret reaches index 0, still inside a Vietnamese run
  -> the character before the window ('g') is Vietnamese
  -> the window is truncated
  -> return without absorbing
```

If we got this wrong in *modify* mode: we would `deleteSurroundingText(-4, 4)`,
turning `"tiếngViệt"` into `"tiếng"` plus a preedit of `"Việt"`, and then
recommit the replacement somewhere else. The document is silently mangled. No
crash, no log, just lost text.

### 8b. Text we did not write

Absorbing only works for words in the commit history. If the user pastes
`"chào"` from the clipboard, the history has no entry for it, so there is
nothing to absorb and the `f` is just a character. That is the correct outcome:
we do not touch text whose provenance we cannot prove.

The corollary is the important one: **when there is no history match, produce
neither a delete nor a commit.** A failed absorb must be a complete no-op.

### 8c. The Firefox duplication

This is the highest-profile failure of the feature, from
[fcitx5-unikey#50](https://github.com/fcitx/fcitx5-unikey/issues/50) (June 2026,
fcitx 5.1.19). In Firefox, in-place editing produced duplicated text
(`chaochào`) while Chromium worked.

The mechanism is worth understanding because it explains why the guard in 8b has
to cover the *pair* of operations, not just the absorb.

`deleteSurroundingText` and `commitString` are **two independent requests** to
the client. A client is free to drop, reorder, or partially apply the first one.
Suppose we believe the host holds our word and we issue:

```
1.  deleteSurroundingText(-4, 4)     client silently drops this
2.  commitString("chao")              client applies this
```

```
host before      "chào"
after step 1     "chào"      (delete ignored)
after step 2     "chàochao"  (commit applied at the caret)
```

The document now contains the word twice. The user sees corrupted text and has
no way to know why. Worse, the caret is in the middle of the duplicate, so the
next keystroke compounds it.

The fix is not to make the delete more polite — it is to make sure we only issue
the pair when we have *proof* that the host text is ours. That proof is the
history match from section 7. With the match as a precondition, a client that
does not cooperate produces a merely-wrong result (`f` inserted as a character)
instead of a corrupted document.

There is a second, weaker mitigation: do not trust the client's surrounding text
across a commit. Track whether we have issued a commit since the last
surrounding-text event, and refuse to absorb until a fresh event arrives.

```cpp
// In the state.
bool awaitingFreshSurrounding_{false};

// In drainCommit(), after a successful commit.
awaitingFreshSurrounding_ = true;

// In absorbFromSurroundingTextIfPending(), first thing.
if (awaitingFreshSurrounding_) {
    return;   // the client's view of the text predates our own edit
}
```

### 8d. Selection

`st.cursor() != st.anchor()`, i.e. `st.selectedText()` non-empty. The user has
selected text; a keypress is meant to replace their selection, not to be
absorbed into a word we supposedly typed. Both unikey paths bail here.

```
host "Xin chào", selection covers "chào" (anchor 4, cursor 8)
user presses 'f'
  -> correct: "Xin f"      (replaces the selection)
  -> wrong:   absorb "chào", delete it, insert "chao"  ->  "Xin chao"
```

### 8e. Password fields

`ic_->capabilityFlags().test(fcitx::CapabilityFlag::Password)`.

fcitx5 does not guarantee that a password field clears or withholds surrounding
text. Reading it is a potential disclosure of the user's password into our
process's logs and buffers, and transforming it is worse. The gate goes at the
top of the absorb, and the debug logging must respect it too — note that
`state.cpp:103` currently logs keys with `VIME_DEBUG`, which is why the existing
code avoids logging in release builds.

### 8f. What is not a bug

**The client advertises the capability but sends nothing.** Then `isValid()` is
false, the absorb is a no-op, and the user gets the preedit-only behaviour. That
is a perfectly good degradation, and it is why the preedit path must keep
working on its own.

---

## 9. Two modes

Once the absorb works, there is a second, independent choice: what to do with
the adopted word.

### Modify mode (what unikey ships as default)

The word leaves the host and becomes our preedit.

```
host "chào", absorb it, user presses 'f'
  deleteSurroundingText(-4, 4)     host: "" (or whatever preceded)
  setClientPreedit("chao")         host displays: "chao" underlined
  -- engine buffer holds "chao"; a later space commits it
```

- The user sees the word, underlined, and can keep editing it.
- Matches our engine model exactly: one active syllable in the buffer.
- Costs a preedit repaint.

### Direct-replace mode

Compute the result and write it back immediately; the engine buffer stays empty
between words.

```
host "chào", absorb it, user presses 'f'
  deleteSurroundingText(-2, 2)     host: "ch"
  commitString("ao")               host: "chao", no preedit at all
```

- No underline, no repaint, nothing to flicker. Feels immediate.
- The engine's buffer is empty almost all the time, so the
  building/dead/`is_phonotactically_valid` machinery is only exercised while
  actively typing.
- This is where the Firefox-class failures bite, because every keystroke is now
  a host mutation.

**Recommendation: modify mode first.** It matches the engine as it exists, and
direct-replace implies a second state model — "the word lives in the host, and
our buffer is a cursor into it" — which is a much larger change than an ABI
addition. It should not be smuggled in as part of an FFI milestone.

---

## 10. The ABI this would need

Current exported surface is 19 functions in `engine/ffi/src/lib.rs`, and **none
of them returns text**. For reference, `VimeRenderState` (in `ffi/src/render.rs`)
already defines a snapshot struct with exactly the fields we would want, plus
`measure()` for the byte/char walk. It is currently unreferenced and is a
natural basis.

Additions, in rough dependency order:

```c
/* 1. Hand the host's text to the engine. Copies; lengths are bytes in,
      offsets are UCS4 characters out, matching fcitx5 exactly. */
void vime_session_set_surrounding(VimeSessionHandle *s,
                                  const char *text, uint32_t len_bytes,
                                  uint32_t cursor_char, uint32_t anchor_char);

/* 2. Try to adopt the word before the caret. Returns chars adopted, 0 = no. */
uint32_t vime_session_absorb_surrounding(VimeSessionHandle *s);

/* 3. Take the pending host mutation, if any. Returns 1 when it filled `out`. */
uint32_t vime_session_take_commit(VimeSessionHandle *s, VimeCommit *out);
```

```c
typedef struct {
    uint32_t struct_size;          /* append-only, append new fields at the end */
    uint32_t delete_before_chars;  /* pass negated to deleteSurroundingText */
    uint32_t delete_after_chars;
    const char *commit_ptr;        /* handle-owned, valid until next mutating call */
    uint32_t commit_len_bytes;
    uint32_t commit_cursor_char;
    uint32_t flags;                /* VIME_COMMIT_FROM_ABSORB, ... */
} VimeCommit;
```

Three rules for the ABI, from the existing code and its mistakes:

1. **`struct_size` first, fields appended, never reordered.** The existing
   `VimeConfig` is a fixed 8 bytes read at hardcoded offsets; it cannot grow.
   Every new struct starts with a size so the core can write only what the
   caller understands.
2. **`uint32_t` for lengths, never `usize`.** All Vietnamese text here is
   single-digit lengths; the ABI should not be hostage to pointer width.
3. **Document the pointer lifetime in the header.** `VimeCommit.commit_ptr`
   points into the handle and dies at the next mutating call. Our existing
   borrowed-pointer model can only be documented, not enforced.

---

## 11. Checklist before writing any of it

- [ ] Confirm `InsertOutcome::Transformed.first_changed` really is an index into
      the *rendered* buffer at the moment of the transform, before relying on it
      for the delete-span arithmetic in §7.
- [ ] Decide the per-client kill switch. Given #50, this feature ships behind a
      config setting, defaulting on only for clients that have been tested.
- [ ] Make `deactivate` commit rather than discard (`INTEGRATION-NOTES.md`
      gap 1) **before** this — otherwise focus loss mid-word still eats input.
- [ ] Teach `showPreedit()` to `setCursor()` in bytes and to honour
      `CapabilityFlag::Preedit` (gaps 2 and 3). Direct-replace mode depends on
      the preedit being right.
- [ ] Decide whether history survives `reset()` and focus loss. It should not
      survive a password field.
- [ ] Check `adapters/nvim` for the same engine-side changes, even though its
      host cannot use the feature.
- [ ] Note that `INTEGRATION-NOTES.md` refers to an older ABI (`vime_parsed`,
      `vime_process_key`, `VimeOutput`) that no longer exists. It is stale on
      names but still correct on findings.

---

## 12. Glossary

| Term | Meaning |
|---|---|
| **Surrounding text** | The application's text in a window around the caret, with caret and selection offsets. The only view we get of text we already committed. |
| **Host text** | The application's real, committed text. Not ours, cannot be undone by us. |
| **Preedit** | Text fcitx5 shows inside the host, underlined, not yet committed. Replaced wholesale on every change. |
| **Commit** | Handing text to the application. Irreversible from our side. |
| **Absorb** | Adopting a host word back into our engine buffer, using commit history to recover its keystrokes. |
| **Rendered** | The parsed Vietnamese form (`chào`). 4 chars. |
| **Raw** | The keystrokes (`chaof`). 5 chars. Different length; never mix indices. |
| **Building / dead** | Whether the buffer still parses as a syllable or has fallen back to verbatim text. |
| **Modify mode** | Adopted word is deleted from the host and shown as preedit. |
| **Direct-replace** | Adopted word is rewritten in place; engine buffer stays empty. |

## Sources

Everything about fcitx5 was read from source, not recalled:

- `fcitx/surroundingtext.h` — `SurroundingText`: `isValid`, `cursor`, `anchor`,
  `text`, `selectedText`, `setText`, `setCursor`, `deleteText`. Doc comments say
  the offsets are in characters.
- `fcitx/inputcontext.h:260-266` — `surroundingText()`, `updateSurroundingText()`.
- `fcitx/inputcontext.h:281-284` — `commitStringWithCursor`, `deleteSurroundingText`.
- `fcitx/inputcontext.h:436-439` — `deleteSurroundingTextImpl`, documented
  "offset of deletion start, in UCS4 char".
- `fcitx-utils/capabilityflags.h:21-48` — `CapabilityFlag` values.
- `fcitx/inputmethodengine.h:90` — `updateSurroundingText` is `FCITXCORE_DEPRECATED`.
- `fcitx5-unikey/src/unikey-im.cpp:196-344` — `rebuildFromSurroundingText`,
  `rebuildPreedit`, the truncated-window guard at `:264-270`, and
  `deleteSurroundingText(-length, length)` at `:342`.
- `fcitx5-unikey/src/unikey-im.cpp:443-449` — the event watcher.
- [fcitx5-unikey#50](https://github.com/fcitx/fcitx5-unikey/issues/50) — the
  Firefox duplication.
- `/usr/include/Fcitx5/Core/fcitx/text.h:34-37` — `Text::cursor()` is in bytes.

Our side:

- `adapters/fcitx5/src/state.cpp` — current key handling, modifier guard,
  `reset()`.
- `adapters/fcitx5/src/engine.cpp` — engine lifecycle, `stateFactory_`.
- `engine/ffi/src/lib.rs` — the 19 exported entry points; no text getter exists.
- `engine/ffi/src/render.rs` — the unused `VimeRenderState` and `measure()`.
- `engine/core/src/session/session.rs:134-158` — `rendered()`,
  `write_rendered_to()`, `raw()`, `write_raw_to()`.
- `engine/core/src/session/session.rs:167-179` — `rendered_cursor()`,
  `raw_cursor()`, both in characters.
