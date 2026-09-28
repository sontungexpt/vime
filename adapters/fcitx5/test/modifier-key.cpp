// Regression test: modified keys must be forwarded to the application, not
// inserted as text.
//
// Two bugs, one in each direction, both invisible to `cargo test`:
//
// 1. Without the guard in `VimeState::keyEvent`, Ctrl+A reached the engine and
//    was inserted: the keysym of Ctrl+A is still `a`, so every shortcut became
//    stray text in the preedit, and the buffer only appeared at commit.
//
// 2. The first fix guarded on `key.hasModifier()`, which counts Shift. fcitx5
//    already strips Shift from a lowercase a-z in `normalize()`, because case
//    is carried by the KEYSYM -- a real frontend sends `FcitxKey_U`, not
//    `FcitxKey_u` plus a Shift state -- so that guard forwarded every
//    capitalised letter and capitalised Vietnamese stopped parsing at all.
//
// The fix is to forward only the modifiers that form shortcuts, and to
// uppercase the character in `toVimeKeyEvent` so the case survives a frontend
// that sends it as state rather than as keysym.
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

    // 4. Shift must still reach the engine. fcitx5 already strips Shift from a
    //    lowercase a-z in normalize(), because case is carried by the KEYSYM: a
    //    real frontend sends FcitxKey_U, not FcitxKey_u plus a Shift state.
    //    Guarding on hasModifier() therefore forwarded every capitalised letter
    //    and capitalised Vietnamese stopped parsing at all.
    //    U-W-F is U+horn+grave in the engine's own uppercase corpus, so it
    //    proves the shape and tone keys were applied.
    {
        for (fcitx::KeySym k : {FcitxKey_U, FcitxKey_W, FcitxKey_F}) {
            fcitx::KeyEvent ev(context.get(), fcitx::Key(k, fcitx::KeyState::Shift), false);
            engine.keyEvent(entry, ev);
        }
        const auto shifted = context->inputPanel().clientPreedit().toString();
        std::cout << "Shift+U,W,F   preedit='" << shifted << "'\n";
        check(shifted == "\u1EEA", "Shift must still be parsed by the engine");
    }

    // 5. CapsLock is a case too, and reaches the engine the same way. Starts
    //    from a reset buffer so it cannot be masked by the Shift case above.
    engine.reset(entry, focus);
    {
        for (fcitx::KeySym k : {FcitxKey_A, FcitxKey_W}) {
            fcitx::KeyEvent ev(context.get(), fcitx::Key(k, fcitx::KeyState::CapsLock), false);
            engine.keyEvent(entry, ev);
        }
        const auto capped = context->inputPanel().clientPreedit().toString();
        std::cout << "CapsLock+A,W  preedit='" << capped << "'\n";
        check(capped == "\u0102", "CapsLock must still be parsed by the engine");
    }

    // 6. Plain Vietnamese input STILL WORKS: "truowngf" -> "trường".
    //    Reset first: the CapsLock keys above carried the CapsLock state, and
    //    the uppercase rule keys off that state on each key, so without a reset
    //    this case would type uppercase and mask what it is checking.
    engine.reset(entry, focus);
    {
        for (auto sym : {FcitxKey_t, FcitxKey_r, FcitxKey_u, FcitxKey_o, FcitxKey_w, FcitxKey_n, FcitxKey_g, FcitxKey_f}) {
            fcitx::Key key(sym);
            fcitx::KeyEvent ev(context.get(), key, false);
            engine.keyEvent(entry, ev);
        }
        const auto pre = context->inputPanel().clientPreedit().toString();
        std::cout << "truowngf preedit='" << pre << "'\n";
        check(pre == "trường", "plain 'truowngf' still parses to 'trường'");
    }

    // 7. Commit on space.
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
