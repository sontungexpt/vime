// Regression test: modified keys must be forwarded to the application, not
// inserted as text.
//
// Before the `key.hasModifier()` guard in `VimeState::keyEvent`, Ctrl+A reached
// the engine and was inserted: the keysym of Ctrl+A is still `a`, so every
// shortcut became stray text in the preedit, and the buffer only appeared at
// commit. `test/modifier-key.cpp` reproduces that by asserting the preedit
// stays empty.
//
// Build (after `cmake --build build`):
//   A=$PWD && R=$A/build/rust-target/release && F=../../../engine/ffi/include
//   g++ -std=c++20 -o /tmp/modifier-key test/modifier-key.cpp \
//       -Itest -I"$A/include" -I"$A/src" -I"$F" \
//       $(pkg-config --cflags Fcitx5Core Fcitx5Utils) \
//       "$A/build/vime.so" -Wl,-rpath,"$A/build" \
//       -L"$R" -lvime -Wl,-rpath,"$R" \
//       $(pkg-config --libs Fcitx5Core Fcitx5Utils)
//   /tmp/modifier-key
//
// Not wired into CMake yet; see INTEGRATION-NOTES.md.
#include "test-input-context.h"
#include "engine.h"
#include "state.h"

#include <fcitx-utils/keysym.h>
#include <fcitx-utils/utf8.h>
#include <iostream>

using namespace vime::fcitx5;

static int failures = 0;
static void check(bool ok, const std::string& what) {
    std::cout << (ok ? "  PASS  " : "  FAIL  ") << what << '\n';
    if (!ok) ++failures;
}

int main() {
    configureTestPaths("vime-modifier-test");
    TestInstance testInstance;
    VimeEngine engine(&testInstance.instance);

    auto context = std::make_unique<TestInputContext>(&testInstance.instance);
    context->setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
    context->focusIn();

    fcitx::InputMethodEntry entry("vime", "Vime", "vi", "vime");
    fcitx::InputContextEvent focus(context.get(), fcitx::EventType::InputContextFocusIn);
    engine.activate(entry, focus);

    // 1. Ctrl+letter must not be inserted as text.
    {
        fcitx::Key key(FcitxKey_a, fcitx::KeyState::Ctrl);
        fcitx::KeyEvent ev(context.get(), key, false);
        engine.keyEvent(entry, ev);
        const auto& pre = context->inputPanel().clientPreedit();
        std::cout << "Ctrl+A  accepted=" << ev.accepted()
                  << " preedit='" << pre.toString() << "'\n";
        check(pre.toString().empty(), "Ctrl+A does not insert text");
        check(!ev.accepted(), "Ctrl+A is not consumed (forwarded to app)");
    }

    // 2. Ctrl+Backspace must also be forwarded, not inserted.
    {
        fcitx::Key key(FcitxKey_BackSpace, fcitx::KeyState::Ctrl);
        fcitx::KeyEvent ev(context.get(), key, false);
        engine.keyEvent(entry, ev);
        check(context->inputPanel().clientPreedit().toString().empty(),
              "Ctrl+BackSpace does not insert text");
    }

    // 3. Alt+letter forwarded too.
    {
        fcitx::Key key(FcitxKey_c, fcitx::KeyState::Alt);
        fcitx::KeyEvent ev(context.get(), key, false);
        engine.keyEvent(entry, ev);
        check(context->inputPanel().clientPreedit().toString().empty(),
              "Alt+C does not insert text");
    }

    // 4. Plain Vietnamese input STILL WORKS: "truowngf" -> "trường"
    {
        for (auto sym : {FcitxKey_t, FcitxKey_r, FcitxKey_u, FcitxKey_o, FcitxKey_w, FcitxKey_n, FcitxKey_g, FcitxKey_f}) {
            fcitx::Key key(sym);
            fcitx::KeyEvent ev(context.get(), key, false);
            engine.keyEvent(entry, ev);
        }
        const auto pre = context->inputPanel().clientPreedit().toString();
        std::cout << "truowngf preedit='" << pre << "'\n";
        check(pre == "trường", "plain 'truowngf' still parses to 'người'");
    }

    // 5. Commit on space.
    {
        fcitx::Key key(FcitxKey_space);
        fcitx::KeyEvent ev(context.get(), key, false);
        engine.keyEvent(entry, ev);
        const auto& commits = context->commits();
        std::string joined;
        for (const auto& c : commits) joined += c;
        std::cout << "commits='" << joined << "'\n";
        check(joined.rfind("trường", 0) == 0, "space commits the composed word");
    }

    std::cout << (failures ? "\nFAILURES\n" : "\nall checks passed\n");
    return failures ? 1 : 0;
}
