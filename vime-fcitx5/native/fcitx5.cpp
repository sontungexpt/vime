#include "fcitx-utils/keysym.h"
#include "fcitx/addonfactory.h"
#include "fcitx/inputmethodengine.h"
#include "fcitx/inputpanel.h"
#include "fcitx/instance.h"
#include "vime_ffi.h"

#include <memory>
#include <unordered_map>
#include <vector>

namespace {

// Translates a fcitx5 `KeyState` into vime-engine's canonical KeyState bits
// (see `VIME_KEY_STATE_*` in `vime_ffi.h`). The engine owns the bit layout;
// this is the only place that knows how fcitx5 describes modifiers.
uint32_t toEngineKeyState(fcitx::KeyStates state)
{
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

class VimeEngine final : public fcitx::InputMethodEngine {
public:
  std::vector<fcitx::InputMethodEntry> listInputMethods() override
  {
    std::vector<fcitx::InputMethodEntry> result;
    auto entry = fcitx::InputMethodEntry("vime", "Vime", "vi", "vime");
    entry.setIcon("input-keyboard").setLabel("Vi");
    result.emplace_back(std::move(entry));
    return result;
  }

  void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override
  {
    auto *ic = event.inputContext();
    auto &state = states_[ic];
    if (!state) {
      state = vime_create();
    }
  }

  void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override
  {
    auto iterator = states_.find(event.inputContext());
    if (iterator != states_.end()) {
      apply(event.inputContext(), vime_reset(iterator->second));
    }
  }

  void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override
  {
    if (event.isRelease()) {
      return;
    }

    auto *ic = event.inputContext();
    auto &state = states_[ic];

    if (!state) {
      state = vime_create();
    }

    const auto key = event.key();
    const auto sym = key.sym();

    VimeKeyEvent input{};

    // Translate Fcitx5 modifier state into vime-engine's canonical KeyState
    // bits (the engine owns the bit layout, not the frontend).
    input.state = toEngineKeyState(key.states());

    switch (sym) {
    case FcitxKey_BackSpace:
      input.key = VIME_KEY_BACKSPACE;
      break;

    case FcitxKey_Delete:
      input.key = VIME_KEY_DELETE;
      break;

    case FcitxKey_Left:
      input.key = VIME_KEY_LEFT;
      break;

    case FcitxKey_Right:
      input.key = VIME_KEY_RIGHT;
      break;

    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
      input.key = VIME_KEY_ENTER;
      break;

    case FcitxKey_Tab:
      input.key = VIME_KEY_TAB;
      break;

    case FcitxKey_Escape:
      input.key = VIME_KEY_ESCAPE;
      break;

    case FcitxKey_space:
      input.key = VIME_KEY_SPACE;
      break;

    default: {
      const auto character = fcitx::Key::keySymToUnicode(sym);

      // Không phải KeySym nào cũng đại diện cho Unicode character.
      if (character == 0) {
        return;
      }

      input.character = character;
      break;
    }
    }

    const auto output = vime_process_key(state, input);

    apply(ic, output);

    if (output.consumed) {
      event.filterAndAccept();
    }
  }

  ~VimeEngine() override
  {
    for (auto &[_, state] : states_) {
      vime_destroy(state);
    }
  }

private:
  static void apply(fcitx::InputContext *ic, VimeOutput output)
  {
    if (output.commit) {
      ic->commitString(output.commit);
      vime_free_string(output.commit);
    }
    if (output.rendered) {
      fcitx::Text text(output.rendered);
      ic->inputPanel().setClientPreedit(text);
      ic->updatePreedit();
      vime_free_string(output.rendered);
    }
  }

  std::unordered_map<fcitx::InputContext *, VimeEngineHandle *> states_;
};

class VimeFactory final : public fcitx::AddonFactory {
public:
  fcitx::AddonInstance *create(fcitx::AddonManager *) override { return new VimeEngine(); }
};

} // namespace

FCITX_ADDON_FACTORY(VimeFactory);
