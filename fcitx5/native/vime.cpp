#include "fcitx/addonfactory.h"
#include "fcitx/inputmethodengine.h"
#include "fcitx/instance.h"
#include "fcitx/inputpanel.h"
#include "fcitx-utils/keysym.h"
#include "vietnamese_fcitx5.h"

#include <memory>
#include <unordered_map>
#include <vector>

namespace {

class VimeEngine final : public fcitx::InputMethodEngine {
public:
    std::vector<fcitx::InputMethodEntry> listInputMethods() override {
        std::vector<fcitx::InputMethodEntry> result;
        auto entry = fcitx::InputMethodEntry("vime", "Vime", "vi", "vime");
        entry.setIcon("input-keyboard").setLabel("Vi");
        result.emplace_back(std::move(entry));
        return result;
    }

    void activate(const fcitx::InputMethodEntry &,
                  fcitx::InputContextEvent &event) override {
        auto *ic = event.inputContext();
        auto &state = states_[ic];
        if (!state) {
            state = vietnamese_fcitx5_create();
        }
    }

    void reset(const fcitx::InputMethodEntry &,
               fcitx::InputContextEvent &event) override {
        auto iterator = states_.find(event.inputContext());
        if (iterator != states_.end()) {
            apply(event.inputContext(),
                  vietnamese_fcitx5_reset(iterator->second));
        }
    }

    void keyEvent(const fcitx::InputMethodEntry &,
                  fcitx::KeyEvent &event) override {
        if (event.isRelease()) {
            return;
        }
        auto *ic = event.inputContext();
        auto &state = states_[ic];
        if (!state) {
            state = vietnamese_fcitx5_create();
        }

        VietnameseFcitx5Output output{};
        const auto key = event.key();
        const auto states = key.states();
        if (states.test(fcitx::KeyState::Ctrl) ||
            states.test(fcitx::KeyState::Alt) ||
            states.test(fcitx::KeyState::Super)) {
            return;
        }

        if (key.sym() == FcitxKey_BackSpace) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_BACKSPACE);
        } else if (key.sym() == FcitxKey_Delete) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_DELETE);
        } else if (key.sym() == FcitxKey_Left) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_LEFT);
        } else if (key.sym() == FcitxKey_Right) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_RIGHT);
        } else if (key.sym() == FcitxKey_Home) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_HOME);
        } else if (key.sym() == FcitxKey_End) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_END);
        } else if (key.sym() == FcitxKey_Return) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_ENTER);
        } else if (key.sym() == FcitxKey_Tab) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_TAB);
        } else if (key.sym() == FcitxKey_Escape) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_ESCAPE);
        } else if (key.sym() == FcitxKey_space) {
            output = vietnamese_fcitx5_process_key(state, VIETNAMESE_KEY_SPACE);
        } else {
            const auto unicode = fcitx::Key::keySymToUnicode(key.sym());
            if (unicode == 0) {
                return;
            }
            output = vietnamese_fcitx5_process_character(state, unicode);
        }

        apply(ic, output);
        if (output.consumed) {
            event.filterAndAccept();
        }
    }

    ~VimeEngine() override {
        for (auto &[_, state] : states_) {
            vietnamese_fcitx5_destroy(state);
        }
    }

private:
    static void apply(fcitx::InputContext *ic, VietnameseFcitx5Output output) {
        if (output.commit) {
            ic->commitString(output.commit);
            vietnamese_fcitx5_free_string(output.commit);
        }
        if (output.rendered) {
            fcitx::Text text(output.rendered);
            ic->inputPanel().setClientPreedit(text);
            ic->updatePreedit();
            vietnamese_fcitx5_free_string(output.rendered);
        }
    }

    std::unordered_map<fcitx::InputContext *, VietnameseFcitx5Engine *> states_;
};

class VimeFactory final : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *) override {
        return new VimeEngine();
    }
};

} // namespace

FCITX_ADDON_FACTORY(VimeFactory);
