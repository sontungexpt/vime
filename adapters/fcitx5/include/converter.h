#pragma once

#include "vime_engine.h"

#include <fcitx-utils/key.h>
#include <fcitx-utils/keysym.h>

#include <optional>

namespace vime::fcitx5 {

static inline uint32_t toVimeKeyState(fcitx::KeyStates state) {
    uint32_t result = 0;

    if (state.test(fcitx::KeyState::Shift)) {
        result |= VIME_KEY_STATE_SHIFT;
    }

    if (state.test(fcitx::KeyState::CapsLock)) {
        result |= VIME_KEY_STATE_CAPS_LOCK;
    }

    if (state.test(fcitx::KeyState::Ctrl)) {
        result |= VIME_KEY_STATE_CTRL;
    }

    if (state.test(fcitx::KeyState::Alt)) {
        result |= VIME_KEY_STATE_ALT;
    }

    if (state.test(fcitx::KeyState::NumLock)) {
        result |= VIME_KEY_STATE_NUM_LOCK;
    }

    if (state.test(fcitx::KeyState::Hyper)) {
        result |= VIME_KEY_STATE_HYPER;
    }

    if (state.test(fcitx::KeyState::Super)) {
        result |= VIME_KEY_STATE_SUPER;
    }

    if (state.test(fcitx::KeyState::Meta)) {
        result |= VIME_KEY_STATE_META;
    }

    return result;
}

static inline std::optional<VimeKeyEvent> toVimeKeyEvent(
    const fcitx::Key& key
) {
    VimeKeyEvent event{};
    event.states = toVimeKeyState(key.states());

    switch (key.sym()) {
    case FcitxKey_BackSpace:
        event.key = VIME_KEY_BACKSPACE;
        return event;

    case FcitxKey_Delete:
        event.key = VIME_KEY_DELETE;
        return event;

    case FcitxKey_Left:
        event.key = VIME_KEY_LEFT;
        return event;

    case FcitxKey_Right:
        event.key = VIME_KEY_RIGHT;
        return event;

    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
        event.key = VIME_KEY_ENTER;
        return event;

    case FcitxKey_Tab:
        event.key = VIME_KEY_TAB;
        return event;

    case FcitxKey_Escape:
        event.key = VIME_KEY_ESCAPE;
        return event;

    case FcitxKey_space:
        event.key = VIME_KEY_SPACE;
        return event;

    default:
        break;
    }

    auto character = fcitx::Key::keySymToUnicode(key.sym());

    if (character == 0) {
        return std::nullopt;
    }

    // The engine reads case from the character, not from the modifier state:
    // its own corpus drives uppercase from literal `U`, `W`, `F`. The keysym
    // alone carries no case, so a capitalised key would arrive lowercase and
    // lose the case the user asked for.
    if (key.states().test(fcitx::KeyState::Shift)
        || key.states().test(fcitx::KeyState::CapsLock)) {
        // ASCII only, matching the engine, which folds key lookups with
        // `to_ascii_lowercase`. Every Telex key is ASCII, so a wider rule would
        // not be more correct here, only more locale-dependent.
        if (character >= 'a' && character <= 'z') {
            character -= 0x20;
        }
    }

    event.character = character;
    return event;
}

}
