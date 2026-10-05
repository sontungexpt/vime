# VIME Fcitx5 Frontend - Production Architecture

> **Status**: Redesigned for concurrent mode/app resolution with real Fcitx5 API.

---

## Core Problem: Concurrent Mode + App Resolution

The current design decides **mode first, then app**. But some keys need **both simultaneously**:

| Key | Mode Decision | App Decision | Conflict |
|-----|---------------|--------------|----------|
| `Tab` | Preedit: insert `\t` | Neovim: send to app | Mode says consume, App says forward |
| `Ctrl+Space` | Any: forward | Terminal: might be IME toggle | Mode says forward, App might consume |
| `BackSpace` | Preedit: delete | Terminal: might be line kill | Different behavior per app |

**Solution**: **Concurrent Resolution** — Mode and App resolve together, not sequentially.

---

## Core Concept: Resolution Context

```cpp
struct ResolutionContext {
    // Input
    fcitx::KeyEvent* event;
    VimeState* state;
    VimeSessionHandle* session;
    fcitx::InputContext* ic;
    
    // Resolved (filled by resolution)
    InputMode mode = InputMode::Preedit;
    AppCapabilities caps;
    std::unique_ptr<ModeStrategy> modeStrategy;
    std::unique_ptr<AppStrategy> appStrategy;
    
    // Decision (mutually exclusive outcomes)
    enum class Decision {
        Consume,      // Engine processes key
        Forward,      // Forward to application
        Transform,    // Engine transforms, then commit
        Absorb,       // Absorb surrounding text first
    } decision = Decision::Consume;
    
    // Output
    VimeInsertResult engineResult;
    std::vector<std::unique_ptr<Effect>> effects;
};
```

---

## Resolution Pipeline (Single Pass)

```cpp
class ResolutionEngine {
public:
    ResolutionEngine(Vime* engine) : engine_(engine) {
        // Register built-in strategies
        modeStrategies_[InputMode::Preedit] = std::make_unique<PreeditStrategy>();
        modeStrategies_[InputMode::Inline] = std::make_unique<InlineStrategy>();
        modeStrategies_[InputMode::SurroundingText] = std::make_unique<SurroundingTextStrategy>();
        modeStrategies_[InputMode::Raw] = std::make_unique<RawStrategy>();
        modeStrategies_[InputMode::Password] = std::make_unique<PasswordStrategy>();
        modeStrategies_[InputMode::Raw] = std::make_unique<RawStrategy>();
        
        // App strategies register themselves
        appRegistry_.registerStrategy("nvim", [](auto& caps) { return std::make_unique<NeovimStrategy>(); });
        appRegistry_.registerPattern("org.gnome.*", [](auto& caps) { return std::make_unique<GnomeStrategy>(); });
        appRegistry_.registerPattern("org.kde.*", [](auto& caps) { return std::make_unique<KdeStrategy>(); });
        // ... more patterns
    }

    void resolve(ResolutionContext& ctx) {
        // 1. Capability detection (once per context)
        ctx.caps = detectCapabilities(ctx.ic);
        
        // 2. Concurrent mode + app resolution
        ctx.mode = resolveMode(ctx);
        ctx.modeStrategy = getModeStrategy(ctx.mode);
        ctx.appStrategy = appRegistry_.create(ctx.caps);
        
        // 3. Joint decision (THE KEY CHANGE)
        ctx.decision = decide(ctx);
        
        // 4. Execute based on decision
        execute(ctx);
    }

private:
    Decision decide(ResolutionContext& ctx) {
        // App gets FIRST say on global shortcuts
        if (ctx.appStrategy->wantsForward(ctx)) {
            return Decision::Forward;
        }
        
        // Mode decides normal processing
        if (ctx.modeStrategy->wantsAbsorb(ctx)) {
            return Decision::Absorb;
        }
        
        // Mode decides transform vs consume
        if (ctx.modeStrategy->wantsTransform(ctx)) {
            return Decision::Transform;
        }
        
        return Decision::Consume;
    }
};
```

---

## Strategy Interfaces (Concrete Fcitx5 API)

### ModeStrategy

```cpp
class ModeStrategy {
public:
    virtual ~ModeStrategy() = default;
    
    // Does this mode want to absorb surrounding text first?
    virtual bool wantsAbsorb(const ResolutionContext& ctx) const {
        return false;  // Only SurroundingTextStrategy returns true
    }
    
    // Does this mode want to transform the key (tone/shape/D-stroke)?
    virtual bool wantsTransform(const ResolutionContext& ctx) const {
        return true;  // Preedit/Inline/SurroundingText transform
    }
    
    // Does this mode want to forward the key to app?
    virtual bool wantsForward(const ResolutionContext& ctx) const {
        return false;  // Only Password/Raw might
    }
    
    // Process the key through engine
    virtual VimeInsertResult processKey(ResolutionContext& ctx) = 0;
    
    // Post-process engine result into effects
    virtual std::vector<std::unique_ptr<Effect>> interpretResult(
        const VimeInsertResult& result,
        const ResolutionContext& ctx) = 0;
};
```

### AppStrategy

```cpp
class AppStrategy {
public:
    virtual ~AppStrategy() = default;
    
    // Does app want to handle this key itself? (Global shortcuts, vim keys, etc.)
    virtual bool wantsForward(const ResolutionContext& ctx) const {
        return false;
    }
    
    // Can app handle this key instead of engine?
    virtual bool canHandleKey(const ResolutionContext& ctx) const {
        return false;
    }
    
    // Modify mode decision based on app
    virtual InputMode adjustMode(InputMode mode, const ResolutionContext& ctx) const {
        return mode;
    }
    
    // Modify effects after mode strategy
    virtual void postProcessEffects(
        std::vector<std::unique_ptr<Effect>>& effects,
        const ResolutionContext& ctx) {}
};
```

---

## Concrete Strategies

### Mode Strategies

```cpp
// Preedit: standard preedit + commit on space
class PreeditStrategy : public ModeStrategy {
public:
    VimeInsertResult processKey(ResolutionContext& ctx) override {
        if (ctx.event->key().sym() == FcitxKey_space) {
            return ctx.bridge->insert(' ');
        }
        return ctx.bridge->insert(ctx.event->key().unicode());
    }
    
    std::vector<std::unique_ptr<Effect>> interpretResult(
        const VimeInsertResult& result, const ResolutionContext& ctx) override {
        std::vector<std::unique_ptr<Effect>> effects;
        
        if (result.kind == VIME_INSERT_INVALID) {
            effects.emplace_back(std::make_unique<ForwardKeyEffect>());
            return effects;
        }
        
        if (auto commit = ctx.bridge->takeCommit()) {
            if (commit->deleteBefore > 0) {
                effects.emplace_back(std::make_unique<DeleteSurroundingEffect>(
                    -static_cast<int>(commit->deleteBefore), commit->deleteBefore));
            }
            if (commit->deleteAfter > 0) {
                effects.emplace_back(std::make_unique<DeleteSurroundingEffect>(
                    0, commit->deleteAfter));
            }
            if (!commit->text.empty()) {
                effects.emplace_back(std::make_unique<CommitEffect>(
                    std::string(commit->text), commit->cursorChar));
            }
        }
        
        // Preedit
        auto rendered = ctx.bridge->getRendered();
        if (rendered.len > 0) {
            int byteOffset = 0;
            auto codepoints = ctx.bridge->getRenderedCodepoints();
            for (size_t i = 0; i < codepoints.len && i < ctx.bridge->renderedCursor(); ++i) {
                byteOffset += utf8_len(codepoints.data[i]);
            }
            effects.emplace_back(std::make_unique<PreeditEffect>(
                std::string(rendered.data, rendered.len), byteOffset));
        } else {
            effects.emplace_back(std::make_unique<ResetPreeditEffect>());
        }
        
        effects.emplace_back(std::make_unique<AcceptKeyEffect>());
        return effects;
    }
};

// InlineStrategy: direct commit, no preedit
class InlineStrategy : public ModeStrategy {
public:
    VimeInsertResult processKey(ResolutionContext& ctx) override {
        return ctx.bridge->insert(ctx.event->key().unicode());
    }
    
    std::vector<std::unique_ptr<Effect>> interpretResult(...) override {
        // Similar to Preedit but NO preedit effect
        // Just CommitEffect + AcceptKeyEffect
    }
};

// SurroundingTextStrategy: absorb + edit
class SurroundingTextStrategy : public ModeStrategy {
public:
    bool wantsAbsorb(const ResolutionContext& ctx) const override {
        return true;
    }
    
    VimeInsertResult processKey(ResolutionContext& ctx) override {
        // Absorption happens BEFORE processKey in ResolutionEngine
        return ctx.bridge->insert(ctx.event->key().unicode());
    }
    
    // Same interpret as Preedit but with absorbed text context
};

// PasswordStrategy: no processing, forward all
class PasswordStrategy : public ModeStrategy {
public:
    bool wantsForward(const ResolutionContext& ctx) const override {
        return true;  // Forward everything
    }
    
    VimeInsertResult processKey(...) { return {VIME_INSERT_INVALID, 0}; }
};

// RawStrategy: pass-through for terminals
class RawStrategy : public ModeStrategy {
public:
    bool wantsForward(const ResolutionContext& ctx) const override {
        // Forward control keys, process printable
        auto ch = ctx.event->key().unicode();
        return ch == 0 || ctx.event->key().states().test(fcitx::KeyState::Ctrl);
    }
};
```

---

### App Strategies

```cpp
// Default: no special behavior
class DefaultAppStrategy : public AppStrategy {};

// Neovim: handle Ctrl+ keys, no preedit
class NeovimStrategy : public AppStrategy {
public:
    bool wantsForward(const ResolutionContext& ctx) override {
        auto states = ctx.event->key().states();
        // Forward Ctrl+ keys to Neovim
        if (states.test(fcitx::KeyState::Ctrl)) return true;
        // Forward Escape in normal mode (heuristic)
        if (ctx.event->key().sym() == FcitxKey_Escape) return true;
        return false;
    }
    
    InputMode adjustMode(InputMode mode, const ResolutionContext& ctx) override {
        // Neovim prefers Inline (no preedit UI)
        if (mode == InputMode::Preedit) return InputMode::Inline;
        return mode;
    }
    
    void postProcessEffects(std::vector<std::unique_ptr<Effect>>& effects, 
                            const ResolutionContext& ctx) override {
        // Ensure no preedit effects for Neovim
        effects.erase(std::remove_if(effects.begin(), effects.end(),
            [](auto& e) { return dynamic_cast<PreeditEffect*>(e.get()); }),
            effects.end());
    }
};

// Terminal: direct commit, surrounding text if available
class TerminalStrategy : public AppStrategy {
public:
    InputMode adjustMode(InputMode mode, const ResolutionContext& ctx) override {
        if (mode == InputMode::Preedit) return InputMode::Inline;
        return mode;
    }
    
    bool wantsForward(const ResolutionContext& ctx) override {
        // Forward Ctrl+ for terminal shortcuts
        if (ctx.event->key().states().test(fcitx::KeyState::Ctrl)) return true;
        return false;
    }
};

// Firefox: autofill guard
class FirefoxStrategy : public AppStrategy {
public:
    void postProcessEffects(std::vector<std::unique_ptr<Effect>>& effects,
                            const ResolutionContext& ctx) override {
        // Guard against autofill duplication
        // If commit text matches recent autofill, skip commit
        for (auto& e : effects) {
            if (auto* ce = dynamic_cast<CommitEffect*>(e.get())) {
                if (looksLikeAutofill(ce->text)) {
                    e = std::make_unique<ForwardKeyEffect>();  // Drop commit
                }
            }
        }
    }
};
```

---

## Resolution Engine Integration

```cpp
class ResolutionEngine {
public:
    void process(ResolutionContext& ctx) {
        // 1. Detect capabilities
        ctx.caps = detectCapabilities(ctx.ic);
        
        // 2. Initial mode from config
        ctx.mode = config_.defaultMode();
        
        // 3. App can adjust mode
        ctx.mode = appRegistry_.create(ctx.caps)->adjustMode(ctx.mode, ctx);
        
        // 4. Create strategies
        ctx.modeStrategy = getModeStrategy(ctx.mode);
        ctx.appStrategy = appRegistry_.create(ctx.caps);
        
        // 5. JOINT DECISION (key innovation)
        if (ctx.appStrategy->wantsForward(ctx)) {
            ctx.decision = Decision::Forward;
        } else if (ctx.modeStrategy->wantsAbsorb(ctx)) {
            ctx.decision = Decision::Absorb;
        } else if (ctx.modeStrategy->wantsTransform(ctx)) {
            ctx.decision = Decision::Transform;
        } else if (ctx.modeStrategy->wantsForward(ctx)) {
            ctx.decision = Decision::Forward;
        } else {
            ctx.decision = Decision::Consume;
        }
        
        // 6. Execute
        execute(ctx);
    }

private:
    void execute(ResolutionContext& ctx) {
        switch (ctx.decision) {
            case Decision::Forward:
                ctx.effects.push_back(std::make_unique<ForwardKeyEffect>());
                break;
                
            case Decision::Absorb:
                ctx.mayAbsorb_ = true;
                // Absorption happens, then fall through to Transform
                [[fallthrough]];
                
            case Decision::Transform:
                ctx.engineResult = ctx.modeStrategy->processKey(ctx);
                ctx.effects = ctx.modeStrategy->interpretResult(ctx.engineResult, ctx);
                ctx.appStrategy->postProcessEffects(ctx.effects, ctx);
                break;
                
            case Decision::Consume:
                ctx.engineResult = ctx.modeStrategy->processKey(ctx);
                ctx.effects = ctx.modeStrategy->interpretResult(ctx.engineResult, ctx);
                ctx.appStrategy->postProcessEffects(ctx.effects, ctx);
                break;
        }
        
        // Apply effects
        for (auto& effect : ctx.effects) {
            effect->execute(ctx.state);
        }
    }
};
```

---

## VimeState Integration

```cpp
void VimeState::keyEvent(fcitx::KeyEvent& event) {
    if (event.isRelease()) return;
    
    ResolutionContext ctx;
    ctx.event = &event;
    ctx.state = this;
    ctx.session = session_;
    ctx.ic = ic_;
    ctx.bridge = std::make_unique<VimeEngineBridge>(session_);
    
    engine_->resolutionEngine()->process(ctx);
    
    // Apply effects (already done in ResolutionEngine)
    // Key already accepted/forwarded via effects
}
```

---

## Key Benefits

| Problem | Solution |
|---------|----------|
| **Tab in Neovim** | `NeovimStrategy::wantsForward` catches `Tab` before mode processes |
| **Ctrl+Space in terminal** | `TerminalStrategy::wantsForward` catches `Ctrl+Space` |
| **Surrounding text + Neovim** | `SurroundingTextStrategy::wantsAbsorb` + `NeovimStrategy::adjustMode → Inline` |
| **Firefox autofill** | `FirefoxStrategy::postProcessEffects` converts `CommitEffect` → `ForwardKeyEffect` |
| **Password field** | `PasswordStrategy::wantsForward` returns `true` for all keys |
| **Ctrl+ keys in vim** | `NeovimStrategy::wantsForward` checks `Ctrl` modifier |

---

## File Structure (Implementation)

```
adapters/fcitx5/
├── include/
│   ├── engine.h
│   ├── state.h
│   ├── resolution_engine.h      # NEW
│   ├── strategies/
│   │   ├── mode_strategy.h      # Base + Preedit/Inline/Surrounding/Raw/Password
│   │   ├── app_strategy.h       # Base + Default/Neovim/Terminal/Firefox
│   │   ├── mode_resolver.h
│   │   ├── app_strategy.h
│   │   ├── capability_detector.h
│   │   └── effect.h
│   ├── bridge.h                 # VimeEngineBridge (FFI adapter)
│   └── config.h
├── src/
│   ├── engine.cpp
│   ├── state.cpp                # Thin: just creates ResolutionContext
│   ├── resolution_engine.cpp    # NEW: core resolution logic
│   ├── strategies/
│   │   ├── mode_strategies.cpp
│   │   ├── app_strategies.cpp
│   │   ├── capability_detector.cpp
│   │   └── effects.cpp
│   ├── bridge.cpp
│   └── factory.cpp
```

---

## Migration Path

| Step | Action |
|------|--------|
| 1 | Implement `ResolutionEngine`, `ModeStrategy`, `AppStrategy` interfaces |
| 2 | Move current `VimeState::keyEvent` logic into `PreeditStrategy` |
| 3 | Add `AppStrategy` with `NeovimStrategy`, `TerminalStrategy` |
| 4 | Wire `ResolutionEngine` into `VimeState::keyEvent` |
| 5 | Add surrounding text absorption (Phase 1) |
| 6 | Add `SurroundingTextStrategy` |
| 7 | Add app strategies for Neovim, Terminal, Firefox |
| 8 | Add config for per-app mode overrides |

---

## Testing Strategy

```cpp
// Unit test each strategy in isolation
TEST(PreeditStrategy, ToneKeyTransforms) {
    ResolutionContext ctx = makeContext("chao");
    auto result = strategy.processKey(ctx);
    EXPECT_EQ(result.kind, VIME_INSERT_TRANSFORMED);
}

TEST(NeovimStrategy, CtrlKeyForwarded) {
    ResolutionContext ctx = makeContextWithApp("nvim", '\t');
    ctx.event->key().states().set(fcitx::KeyState::Ctrl);
    EXPECT_TRUE(appStrategy->wantsForward(ctx));
}

TEST(SurroundingTextStrategy, AbsorbChao) {
    // Setup surrounding text "chào"
    ResolutionContext ctx = makeContextWithSurrounding("chào", 4);
    EXPECT_TRUE(strategy.wantsAbsorb(ctx));
}
```

---

*This design replaces sequential mode→app resolution with concurrent resolution where both mode and app strategies participate in the decision simultaneously.*