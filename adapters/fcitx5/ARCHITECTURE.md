# VIME Fcitx5 Frontend Architecture

## High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FCITX5 INPUT CONTEXT                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │ KeyEvent     │→ │ Pipeline     │→ │ Stage 1:     │→ │ Stage 2:     │    │
│  │ (raw)        │  │ Dispatcher   │  │ Global       │  │ Mode         │    │
│  └──────────────┘  └──────────────┘  │ Shortcuts    │  │ Resolution   │    │
│                                      └──────────────┘  └──────┬───────┘    │
│                                                                │            │
│                                      ┌──────────────┐         ▼            │
│                                      │ Stage 3:     │→ ┌──────────────┐   │
│                                      │ Application  │  │ Stage 4:     │   │
│                                      │ Capability   │  │ Mode-Specific│   │
│                                      └──────────────┘  │ Strategy     │   │
│                                                         └──────┬───────┘   │
│                                                                │            │
│                                      ┌──────────────┐         ▼            │
│                                      │ Stage 5:     │→ ┌──────────────┐   │
│                                      │ App-Specific │  │ Stage 6:     │   │
│                                      │ Overlay      │  │ Vietnamese   │   │
│                                      └──────────────┘  │ Engine (Rust)│   │
│                                                         └──────┬───────┘   │
│                                                                │            │
│                                      ┌──────────────┐         ▼            │
│                                      │ Stage 7:     │→ ┌──────────────┐   │
│                                      │ Result       │  │ Stage 8:     │   │
│                                      │ Interpretation │  │ Fcitx5       │   │
│                                      └──────────────┘  │ Effect       │   │
│                                                         │ Executor     │   │
│                                                         └──────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Component Responsibilities & Design Patterns

| Component | Pattern | Responsibility |
|-----------|---------|----------------|
| **PipelineDispatcher** | Pipeline / Chain of Responsibility | Orchestrates stages, passes context, short-circuits on handled |
| **GlobalShortcutFilter** | Strategy (per-shortcut) | Filters Ctrl/Alt/Meta/Super/Hyper combos; never reaches engine |
| **ModeResolver** | Strategy + Factory | Selects active `InputMode` based on config + application |
| **CapabilityDetector** | Strategy | Reads `InputContext` capability flags, surrounding text, app info |
| **ModeStrategy** | Strategy | Per-mode key handling logic (preedit, commit, inline, etc.) |
| **AppStrategy** | Strategy + Decorator | Application-specific overrides layered on mode strategy |
| **VimeEngineBridge** | Adapter | Translates Fcitx5 types → Rust FFI types; calls Rust session |
| **ResultInterpreter** | Strategy | Maps Rust `VimeInsertResult` → Fcitx5 actions (commit, preedit, delete, cursor) |
| **EffectExecutor** | Command | Executes Fcitx5 side effects (commitString, deleteSurroundingText, setPreedit) |

---

## Event-Processing Pipeline

```cpp
// Core pipeline data structure - passes through all stages
struct KeyEventContext {
    fcitx::KeyEvent* event;
    VimeState* state;
    VimeSessionHandle* session;
    InputContext* ic;
    
    // Filled by stages
    bool handled = false;           // Short-circuit flag
    bool forwardToApp = false;      // Let Fcitx5 forward key
    InputMode mode = InputMode::Preedit;
    AppCapabilities caps;
    std::unique_ptr<ModeStrategy> modeStrategy;
    std::unique_ptr<AppStrategy> appStrategy;
    VimeInsertResult engineResult;
    std::vector<Effect> effects;    // Commands to execute
};
```

### Pipeline Stages

```cpp
class PipelineDispatcher {
public:
    PipelineDispatcher(Vime* engine) : engine_(engine) {
        stages_.push_back(std::make_unique<GlobalShortcutFilter>());
        stages_.push_back(std::make_unique<ModeResolutionStage>(engine));
        stages_.push_back(std::make_unique<CapabilityDetectionStage>());
        stages_.push_back(std::make_unique<ModeStrategyStage>());
        stages_.push_back(std::make_unique<AppOverlayStage>());
        stages_.push_back(std::make_unique<EngineExecutionStage>());
        stages_.push_back(std::make_unique<ResultInterpretationStage>());
        stages_.push_back(std::make_unique<EffectExecutionStage>());
    }

    void process(KeyEventContext& ctx) {
        for (auto& stage : stages_) {
            stage->execute(ctx);
            if (ctx.handled || ctx.forwardToApp) break;
        }
    }

private:
    Vime* engine_;
    std::vector<std::unique_ptr<PipelineStage>> stages_;
};
```

---

## Mode Selection

```cpp
enum class InputMode {
    Preedit,           // Default: preedit + commit on space
    Inline,            // Direct commit, no preedit
    SurroundingText,   // Absorb committed words for tone editing
    Password,          // Pass-through, no processing
    Raw,               // Pass-through for terminals/vim
    None               // Disabled
};

class ModeResolver {
public:
    InputMode resolve(const AppCapabilities& caps, const Config& config) {
        // 1. Password field check
        if (caps.isPassword) return InputMode::Password;
        
        // 2. Terminal/Vim detection
        if (caps.isTerminal || caps.isNeovim) {
            return config.terminalMode();  // Configurable: Raw / Inline
        }
        
        // 3. Surrounding text capability
        if (caps.hasSurroundingText && config.useSurroundingText()) {
            return InputMode::SurroundingText;
        }
        
        // 4. Preedit capability
        if (caps.hasPreedit) return InputMode::Preedit;
        
        // 5. Fallback
        return InputMode::Inline;
    }
};
```

---

## Application-Specific Strategy Selection

```cpp
struct AppCapabilities {
    // Capability flags
    bool hasPreedit = false;
    bool hasSurroundingText = false;
    bool hasPassword = false;
    
    // Application identification
    std::string programName;      // e.g., "gnome-terminal", "firefox", "nvim"
    std::string applicationId;    // Desktop file ID
    InputPurpose inputPurpose;
    InputHints inputHints;
    
    // Derived
    bool isTerminal = false;
    bool isNeovim = false;
    bool isBrowser = false;
    bool isElectron = false;
};

class AppStrategyRegistry {
public:
    using Factory = std::function<std::unique_ptr<AppStrategy>(const AppCapabilities&)>;
    
    void registerStrategy(std::string_view appId, Factory factory) {
        factories_[appId] = std::move(factory);
    }
    
    std::unique_ptr<AppStrategy> create(const AppCapabilities& caps) {
        // Exact match first
        if (auto it = factories_.find(caps.applicationId); it != factories_.end()) {
            return it->second(caps);
        }
        // Pattern match (e.g., "org.gnome.*")
        for (auto& [pattern, factory] : patternFactories_) {
            if (fnmatch(pattern.c_str(), caps.applicationId.c_str(), 0) == 0) {
                return factory(caps);
            }
        }
        return std::make_unique<DefaultAppStrategy>(caps);
    }
    
private:
    std::unordered_map<std::string, Factory> factories_;
    std::vector<std::pair<std::string, Factory>> patternFactories_;
};

// Example strategies
class DefaultAppStrategy : public AppStrategy {
    // Standard behavior
};

class NeovimStrategy : public AppStrategy {
    // No preedit, no surrounding text, direct commit
    // Handles Ctrl+ keys specially for vim compatibility
};

class TerminalStrategy : public AppStrategy {
    // Direct commit, minimal preedit
    // Surrounding text if available (foot, kitty, wezterm)
};

class FirefoxStrategy : public AppStrategy {
    // Extra guard for autofill duplication
    // Surrounding text with truncated-window detection
};
```

---

## Capability Detection

```cpp
class CapabilityDetector {
public:
    AppCapabilities detect(fcitx::InputContext* ic) {
        AppCapabilities caps;
        
        auto flags = ic->capabilityFlags();
        caps.hasPreedit = flags.test(fcitx::CapabilityFlag::Preedit);
        caps.hasSurroundingText = flags.test(fcitx::CapabilityFlag::SurroundingText);
        caps.hasPassword = flags.test(fcitx::CapabilityFlag::Password);
        
        // Get program name (X11/Wayland)
        caps.programName = getProgramName(ic);
        caps.applicationId = getApplicationId(ic);
        
        // Heuristics
        caps.isTerminal = isTerminalApp(caps.programName);
        caps.isNeovim = caps.programName.find("nvim") != std::string::npos;
        caps.isBrowser = isBrowserApp(caps.programName);
        caps.isElectron = isElectronApp(caps.programName);
        
        return caps;
    }
    
private:
    std::string getProgramName(fcitx::InputContext* ic) {
        // fcitx5 stores this on the InputContext
        return ic->programName().toStdString();
    }
    
    bool isTerminalApp(std::string_view name) {
        static const std::vector<std::string> terminals = {
            "gnome-terminal", "konsole", "xterm", "alacritty", "kitty",
            "wezterm", "foot", "terminator", "tilix", "terminology"
        };
        return std::any_of(terminals.begin(), terminals.end(),
            [&](auto& t) { return name.find(t) != std::string::npos; });
    }
};
```

---

## Rust VIME Session Integration

```cpp
// Adapter-only type - bridges Fcitx5 → Rust FFI
class VimeEngineBridge {
public:
    explicit VimeEngineBridge(VimeSessionHandle* session) : session_(session) {}
    
    // Key processing
    VimeInsertResult insert(uint32_t ch) {
        return vime_session_insert(session_, ch);
    }
    
    bool backspace() { return vime_session_backspace(session_); }
    bool del() { return vime_session_delete(session_); }
    bool moveLeft(uint32_t n = 1) { return vime_session_move_cursor_left(session_, n); }
    bool moveRight(uint32_t n = 1) { return vime_session_move_cursor_right(session_, n); }
    
    // State queries
    size_t renderedLen() { return vime_session_get_rendered_codepoint_len(session_); }
    size_t rawLen() { return vime_session_get_raw_codepoint_len(session_); }
    bool isValid() { return vime_session_is_phonotactically_valid(session_); }
    
    // Rendering (borrowed pointers, valid until next mutating call)
    VimeStringView getRendered() { return vime_session_get_rendered(session_); }
    VimeCodepointView getRenderedCodepoints() { return vime_session_get_rendered_codepoints(session_); }
    VimeStringView getRaw() { return vime_session_get_raw(session_); }
    VimeCodepointView getRawCodepoints() { return vime_session_get_raw_codepoints(session_); }
    
    // Surrounding text (for absorb mode)
    void setSurrounding(std::string_view text, uint32_t cursor, uint32_t anchor) {
        vime_session_set_surrounding(session_, text.data(), text.size(), cursor, anchor);
    }
    uint32_t absorbSurrounding() { return vime_session_absorb_surrounding(session_); }
    
    // Commit handling
    struct Commit {
        uint32_t deleteBefore = 0;
        uint32_t deleteAfter = 0;
        std::string_view text;
        uint32_t cursorChar = 0;
    };
    std::optional<Commit> takeCommit() {
        VimeCommit c{sizeof(VimeCommit)};
        if (vime_session_take_commit(session_, &c)) {
            return Commit{c.delete_before_chars, c.delete_after_chars,
                         std::string_view(c.commit_ptr, c.commit_len_bytes),
                         c.commit_cursor_char};
        }
        return std::nullopt;
    }
    
    void reset() { vime_session_reset(session_); }

private:
    VimeSessionHandle* session_;
};
```

---

## Result Interpretation → Fcitx5 Effects

```cpp
// Command pattern - each effect is a self-contained operation
struct Effect {
    virtual ~Effect() = default;
    virtual void execute(VimeState* state) = 0;
};

struct CommitEffect : Effect {
    std::string text;
    uint32_t cursorChar = 0;
    void execute(VimeState* state) override {
        state->ic()->commitStringWithCursor(text, cursorChar);
        state->ic()->inputPanel().setClientPreedit(fcitx::Text(""));
        state->ic()->updatePreedit();
    }
};

struct DeleteSurroundingEffect : Effect {
    int offset;  // negative = before caret
    uint32_t size;
    void execute(VimeState* state) override {
        state->ic()->deleteSurroundingText(offset, size);
    }
};

struct PreeditEffect : Effect {
    std::string text;
    int cursorByteOffset = -1;
    void execute(VimeState* state) override {
        fcitx::Text t(text);
        if (cursorByteOffset >= 0) t.setCursor(cursorByteOffset);
        
        if (state->ic()->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
            state->ic()->inputPanel().setClientPreedit(t);
        } else {
            state->ic()->inputPanel().setPreedit(t);
        }
        state->ic()->updatePreedit();
    }
};

struct ResetPreeditEffect : Effect {
    void execute(VimeState* state) override {
        state->ic()->inputPanel().setClientPreedit(fcitx::Text(""));
        state->ic()->updatePreedit();
    }
};

struct ForwardKeyEffect : Effect {
    void execute(VimeState* state) override {
        // Do NOT call filterAndAccept() - let Fcitx5 forward
    }
};

struct AcceptKeyEffect : Effect {
    void execute(VimeState* state) override {
        state->event()->filterAndAccept();
    }
};
```

---

## Result Interpreter

```cpp
class ResultInterpreter {
public:
    std::vector<std::unique_ptr<Effect>> interpret(
        const VimeInsertResult& result,
        const VimeEngineBridge& bridge,
        const AppCapabilities& caps,
        InputMode mode) 
    {
        std::vector<std::unique_ptr<Effect>> effects;
        
        if (result.kind == VIME_INSERT_INVALID) {
            effects.push_back(std::make_unique<ForwardKeyEffect>());
            return effects;
        }
        
        // Handle commit from engine
        if (auto commit = bridge.takeCommit()) {
            if (commit->deleteBefore > 0) {
                effects.push_back(std::make_unique<DeleteSurroundingEffect>(
                    -static_cast<int>(commit->deleteBefore), commit->deleteBefore));
            }
            if (commit->deleteAfter > 0) {
                effects.push_back(std::make_unique<DeleteSurroundingEffect>(
                    0, commit->deleteAfter));
            }
            if (!commit->text.empty()) {
                effects.push_back(std::make_unique<CommitEffect>(
                    std::string(commit->text), commit->cursorChar));
            }
        }
        
        // Handle preedit / cursor
        auto rendered = bridge.getRendered();
        if (rendered.len > 0) {
            // Convert char cursor to byte cursor
            auto codepoints = bridge.getRenderedCodepoints();
            int byteOffset = 0;
            for (size_t i = 0; i < codepoints.len && i < renderedCursorChar; ++i) {
                byteOffset += utf8_len(codepoints.data[i]);
            }
            effects.push_back(std::make_unique<PreeditEffect>(
                std::string(rendered.data, rendered.len), byteOffset));
        } else {
            effects.push_back(std::make_unique<ResetPreeditEffect>());
        }
        
        effects.push_back(std::make_unique<AcceptKeyEffect>());
        return effects;
    }
    
private:
    static int utf8_len(uint32_t cp) {
        if (cp < 0x80) return 1;
        if (cp < 0x800) return 2;
        if (cp < 0x10000) return 3;
        return 4;
    }
};
```

---

## VimeState - Refactored

```cpp
// include/state.h
#pragma once

#include "vime_engine.h"
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/event.h>
#include <memory>

namespace vime::fcitx5 {

class Vime;
class PipelineDispatcher;

class VimeState final : public fcitx::InputContextProperty {
public:
    VimeState(Vime* engine, fcitx::InputContext* ic);
    ~VimeState() override;

    void keyEvent(fcitx::KeyEvent& event);
    void reset();
    void setMode(InputMode mode);           // For mode switcher UI
    InputMode currentMode() const { return mode_; }

private:
    void absorbFromSurroundingTextIfPending();
    void pushSurroundingTextToEngine();
    
    Vime* engine_;
    fcitx::InputContext* ic_;
    VimeSessionHandle* session_;
    std::unique_ptr<VimeEngineBridge> bridge_;
    std::unique_ptr<PipelineDispatcher> pipeline_;
    
    InputMode mode_ = InputMode::Preedit;
    AppCapabilities capabilities_;
    
    // Surrounding text state
    bool mayAbsorb_ = false;
    bool absorbed_ = false;
    bool awaitingFreshSurrounding_ = false;
    
    // Event watcher for surrounding text updates
    fcitx::EventWatcher surroundingTextWatcher_;
};

} // namespace vime::fcitx5
```

---

## VimeEngine - Refactored

```cpp
// include/engine.h
#pragma once

#include "state.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <vector>

namespace vime::fcitx5 {

class Vime final : public fcitx::InputMethodEngine {
public:
    explicit Vime(fcitx::Instance* instance);
    ~Vime() override;

    std::vector<fcitx::InputMethodEntry> listInputMethods() override;
    void activate(const fcitx::InputMethodEntry&, fcitx::InputContextEvent&) override;
    void deactivate(const fcitx::InputMethodEntry&, fcitx::InputContextEvent&) override;
    void reset(const fcitx::InputMethodEntry&, fcitx::InputContextEvent&) override;
    void keyEvent(const fcitx::InputMethodEntry&, fcitx::KeyEvent&) override;

    fcitx::Instance* instance() const { return instance_; }
    VimeSessionFactoryHandle* sessionFactory() const { return sessionFactory_; }
    AppStrategyRegistry& appStrategyRegistry() { return appStrategyRegistry_; }
    Config& config() { return config_; }

private:
    fcitx::Instance* instance_;
    fcitx::FactoryFor<VimeState> stateFactory_;
    VimeSessionFactoryHandle* sessionFactory_;
    
    // New components
    Config config_;
    AppStrategyRegistry appStrategyRegistry_;
    CapabilityDetector capabilityDetector_;
    std::vector<fcitx::EventWatcher> eventWatchers_;
};

} // namespace vime::fcitx5
```

---

## File/Class Structure

```
adapters/fcitx5/
├── include/
│   ├── engine.h              # Vime (InputMethodEngine)
│   ├── state.h               # VimeState (InputContextProperty)
│   ├── pipeline.h            # PipelineDispatcher, KeyEventContext, PipelineStage
│   ├── strategies/
│   │   ├── global_shortcut_filter.h
│   │   ├── mode_resolver.h
│   │   ├── mode_strategy.h   # Base + PreeditStrategy, InlineStrategy, SurroundingStrategy
│   │   ├── app_strategy.h    # Base + DefaultApp, NeovimApp, TerminalApp, BrowserApp
│   │   ├── capability_detector.h
│   │   ├── result_interpreter.h
│   │   └── effect.h          # Effect base + concrete effects
│   ├── bridge.h              # VimeEngineBridge (FFI adapter)
│   └── config.h              # Configuration (modes, per-app rules)
├── src/
│   ├── engine.cpp
│   ├── state.cpp
│   ├── pipeline.cpp
│   ├── strategies/
│   │   ├── global_shortcut_filter.cpp
│   │   ├── mode_resolver.cpp
│   │   ├── mode_strategies.cpp
│   │   ├── app_strategies.cpp
│   │   ├── capability_detector.cpp
│   │   ├── result_interpreter.cpp
│   │   └── effects.cpp
│   ├── bridge.cpp
│   └── factory.cpp
├── CMakeLists.txt
└── vime.conf
```

---

## Ownership & Lifetime

| Object | Owner | Lifetime |
|--------|-------|----------|
| `Vime` (engine) | Fcitx5 AddonManager | Process lifetime |
| `VimeState` | Fcitx5 InputContextManager (per-InputContext) | InputContext lifetime |
| `VimeSessionHandle` | `VimeState` | Same as VimeState |
| `PipelineDispatcher` | `VimeState` | Same as VimeState |
| `ModeStrategy` / `AppStrategy` | `PipelineDispatcher` | Reused across events (created once per context) |
| `VimeEngineBridge` | `VimeState` | Same as VimeState |
| `Effect` objects | Created per-event, executed immediately | Single event |

**Key point**: Strategies are created **once** when `VimeState` is created (or when mode/app changes), not per-keystroke. The `PipelineDispatcher` holds `std::unique_ptr<ModeStrategy>` and `std::unique_ptr<AppStrategy>` that are reused.

---

## Performance Optimizations

| Concern | Solution |
|---------|----------|
| Per-keystroke allocations | Effects created on stack via `std::vector<std::unique_ptr<Effect>>` (small buffer optimization); strings use SSO |
| Strategy lookup | Mode/App strategies resolved once at context creation or mode change, cached |
| Capability detection | Cached in `VimeState::capabilities_`; refreshed only on `SurroundingTextUpdated` event |
| UTF-8 conversion | Done once at FFI boundary; `VimeStringView` / `VimeCodepointView` are borrowed |
| Surrounding text | Copied once per event via `setSurrounding()`; engine caches |
| Virtual dispatch | Only at strategy boundaries (mode/app); hot path uses function pointers or `std::function` with small buffer |

---

## Adding a New Application-Specific Strategy

**Without changing existing pipeline code:**

```cpp
// 1. Create the strategy (e.g., src/strategies/app_myapp.cpp)
class MyAppStrategy : public AppStrategy {
public:
    explicit MyAppStrategy(const AppCapabilities& caps) : AppStrategy(caps) {}
    
    void onKeyEvent(KeyEventContext& ctx) override {
        // Custom behavior for this app
        if (ctx.event->key().sym() == FcitxKey_Tab) {
            // Handle Tab specially
            ctx.engineResult = bridge_.insert('\t');
            ctx.handled = true;
        } else {
            AppStrategy::onKeyEvent(ctx);  // Fallback to default
        }
    }
    
    std::vector<std::unique_ptr<Effect>> interpretResult(const VimeInsertResult& result) override {
        // Custom interpretation
        auto effects = AppStrategy::interpretResult(result);
        // Add/modify effects
        return effects;
    }
};

// 2. Register in engine initialization (engine.cpp constructor)
appStrategyRegistry_.registerStrategy("com.myapp.MyApp", 
    [](const AppCapabilities& caps) { 
        return std::make_unique<MyAppStrategy>(caps); 
    });

// 3. Or pattern-match for a family
appStrategyRegistry_.registerPattern("org.gnome.*",
    [](const AppCapabilities& caps) { 
        return std::make_unique<GnomeAppStrategy>(caps); 
    });
```

**Zero changes to pipeline, mode strategies, or core logic.**

---

## Concrete Example: Typing `chaof` + `space` in Preedit Mode

```
1. KeyEvent('c') arrives
   ├─ GlobalShortcutFilter: no modifiers → pass
   ├─ ModeResolution: Preedit (has preedit, no surrounding)
   ├─ CapabilityDetection: cached from activate()
   ├─ ModeStrategy(Preedit): insert('c') → engine
   ├─ AppStrategy(Default): no override
   ├─ EngineExecution: vime_session_insert('c') → VIME_INSERT_EXTENDED
   ├─ ResultInterpretation: rendered="c", cursor=1
   ├─ EffectExecutor: PreeditEffect("c", cursor=1) + AcceptKeyEffect
   └─ event.filterAndAccept()

2. KeyEvent('h') → same → rendered="ch"

3. KeyEvent('a') → same → rendered="cha"

4. KeyEvent('o') → same → rendered="chao"

5. KeyEvent('f') (tone key)
   ├─ EngineExecution: vime_session_insert('f') → VIME_INSERT_TRANSFORMED
   ├─ ResultInterpretation: rendered="chào", cursor=3 (char), byte=4
   ├─ EffectExecutor: PreeditEffect("chào", cursor=4) + AcceptKeyEffect

6. KeyEvent(space)
   ├─ EngineExecution: vime_session_insert(' ') → VIME_INSERT_EXTENDED (commit)
   ├─ bridge.takeCommit() → {deleteBefore=0, deleteAfter=0, text="chào ", cursor=5}
   ├─ EffectExecutor: 
   │    CommitEffect("chào ", 5)
   │    ResetPreeditEffect()
   │    AcceptKeyEffect()
   └─ ic.commitStringWithCursor("chào ", 5)
```

---

## Summary of Key Design Decisions

1. **Pipeline + Strategy**: Clear separation of "what runs when" (pipeline) vs "how it behaves" (strategy)
2. **Zero per-keystroke allocations**: Strategies reused; effects use SSO; borrowed FFI views
3. **Rust core isolation**: Only `VimeEngineBridge` touches FFI; no Fcitx5 types in Rust
4. **Real Fcitx5 APIs**: Uses `capabilityFlags()`, `surroundingText()`, `commitStringWithCursor()`, `deleteSurroundingText()`, `setClientPreedit()`/`setPreedit()`, `inputPanel()`
5. **Extensible**: New modes = new `ModeStrategy`; new apps = new `AppStrategy` registered in factory
6. **Production-ready**: Handles password fields, surrounding text truncation, Firefox duplication, cursor in bytes, capability fallbacks

This architecture directly addresses all requirements while building on the existing `SURROUNDING-TEXT.md` and `INTEGRATION-NOTES.md` design work.