#include "state.h"
#include "engine.h"
#include "log.h"

#include <fcitx-utils/keysym.h>
#include <fcitx/inputpanel.h>

#include <iomanip>
#include <string>

namespace vime::fcitx5 {

VimeState::VimeState(Vime *engine, fcitx::InputContext *ic)
    : engine_(engine), ic_(ic), session_(vime_session_create(engine_->sessionFactory()))
{
}

VimeState::~VimeState()
{
  if (session_) {
    vime_session_destroy(session_);
    session_ = nullptr;
  }
}

// Helper: compute byte offset of cursor in UTF-8 string
static size_t cursorByteOffset(const char *text, size_t codepointIndex) {
    if (!text || codepointIndex == 0) {
        return 0;
    }
    size_t byteOffset = 0;
    const unsigned char *p = reinterpret_cast<const unsigned char *>(text);
    size_t charIndex = 0;
    while (codepointIndex > 0 && *p) {
        size_t charLen = 1;
        if ((*p & 0x80) == 0x00) charLen = 1;
        else if ((*p & 0xE0) == 0xC0) charLen = 2;
        else if ((*p & 0xF0) == 0xE0) charLen = 3;
        else if ((*p & 0xF8) == 0xF0) charLen = 4;

        if (charIndex == codepointIndex) {
            break;
        }
        byteOffset += charLen;
        p += charLen;
        codepointIndex--;
    }
    return byteOffset;
}

// Show preedit with proper cursor position
void VimeState::showPreedit() {
    if (!ic_ || !session_) {
        return;
    }

    VimeStringView rendered = vime_session_get_rendered(session_);
    if (rendered.len == 0) {
        ic_->inputPanel().setClientPreedit(fcitx::Text(""));
        ic_->updatePreedit();
        return;
    }

    size_t cursorChars = vime_session_get_rendered_cursor(session_);
    size_t cursorBytes = cursorByteOffset(rendered.data, cursorChars);

    fcitx::Text preeditText(std::string(rendered.data, rendered.len));
    preeditText.setCursor(static_cast<int>(cursorBytes));

    if (ic_->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
        ic_->inputPanel().setClientPreedit(preeditText);
    } else {
        ic_->inputPanel().setPreedit(preeditText);
    }
    ic_->updatePreedit();
}

void VimeState::keyEvent(fcitx::KeyEvent &event)
{
    if (event.isRelease() || !session_) {
        return;
    }

    const auto key = event.key();

    // Shift and CapsLock change a key's case, not its meaning, so a letter
    // under either still has to reach the engine: uppercase is how Telex writes
    // a shape (`W` for `w`) and how a caller asks for a capital. Only the
    // modifiers that combine with a key into a shortcut are forwarded.
    //
    // The keysym of Ctrl+A is still `a`, so without this a shortcut would
    // reach the engine and be inserted as text. That also corrupted whatever
    // was typed next, since the stray letter stayed in the buffer.
    const auto states = key.states();
    if (states.test(fcitx::KeyState::Ctrl) || states.test(fcitx::KeyState::Alt)
        || states.test(fcitx::KeyState::Super) || states.test(fcitx::KeyState::Hyper)
        || states.test(fcitx::KeyState::Meta)) {
        return;
    }

    VIME_IF_DEV({
        VIME_DEBUG()
            << "key="
            << fcitx::Key::keySymToString(key.sym())
            << " states=0x"
            << std::hex
            << key.states()
            << std::dec;
    });

    switch (key.sym()) {
    case FcitxKey_BackSpace:
        vime_session_backspace(session_);
        break;

    case FcitxKey_Delete:
        vime_session_delete(session_);
        break;
    case FcitxKey_Left:
        vime_session_move_cursor_left(session_, 1);
        break;
    case FcitxKey_Right:
        vime_session_move_cursor_right(session_, 1);
        break;
    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
        break;

    case FcitxKey_Tab:
        break;

    case FcitxKey_Escape:
        break;

    case FcitxKey_space:
        break;

    default:
        break;
    }

    uint32_t character = fcitx::Key::keySymToUnicode(key.sym());

    if (character == 0) {
        return;
    }

    VimeInsertResult result = vime_session_insert(session_, character);

    // Show preedit after every key that produces output
    if (result.kind != VIME_INSERT_INVALID) {
        showPreedit();
    }

    // Do not show any key in release mode because user can type password or something like that
    // So if we require them to send some log to debug maybe you will be busted :))
    VIME_IF_DEV({
        VIME_DEBUG()
            << "vime_process_key"
            << " action="
            << static_cast<int>(result.kind);
    });


    event.filterAndAccept();
}

void VimeState::reset()
{
    if (!session_) {
        return;
    }

    vime_session_reset(session_);

    // The buffer is empty, so the preedit must go with it. `vime_reset` has no
    // action to dispatch -- it consumes no key and commits nothing -- so the
    // repaint is this method's own responsibility rather than `apply`'s.
    ic_->inputPanel().setClientPreedit(fcitx::Text(""));
    ic_->updatePreedit();
}

} // namespace vime::fcitx5