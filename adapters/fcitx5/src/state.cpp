#include "state.h"
#include "engine.h"
#include "converter.h"
#include "log.h"

#include <fcitx-utils/keysym.h>
#include <fcitx/inputpanel.h>

#include <iomanip>
#include <string>

namespace vime::fcitx5 {

VimeState::VimeState(
    VimeEngine *engine,
    fcitx::InputContext *ic)
    : engine_(engine)
    , ic_(ic)
    , handle_(vime_create())
{
}

VimeState::~VimeState()
{
    if (handle_) {
        vime_destroy(handle_);
        handle_ = nullptr;
    }
}


void VimeState::keyEvent(fcitx::KeyEvent &event)
{
    if (event.isRelease() || !handle_) {
        return;
    }

    const auto key = event.key();
    const auto vimeKeyEvent = toVimeKeyEvent(key);

    if (!vimeKeyEvent) {
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

    const auto output = vime_process_key(
        handle_,
        *vimeKeyEvent
    );

    // Do not show any key in release mode because user can type password or something like that
    // So if we require them to send some log to debug maybe you will be busted :))
    VIME_IF_DEV({
        VIME_DEBUG()
            << "vime_process_key"
            << " action="
            << static_cast<int>(output.action);
    });

    if (output.action == VIME_ACTION_FORWARD) {
        return;
    }

    apply(output);
    event.filterAndAccept();
}

void VimeState::reset()
{
    if (!handle_) {
        return;
    }

    const auto output = vime_reset(handle_);
    apply(output);
}

void VimeState::setInputMethod(VimeInputMethod method)
{
    if (!handle_) {
        return;
    }

    vime_set_input_method(handle_, method);
}

void VimeState::apply(VimeOutput output)
{
    if (!ic_) {
        return;
    }

    switch (output.action) {
    case VIME_ACTION_COMMIT:
        if (output.commit) {
            VIME_INFO()
                << "commit: "
                << output.commit;

            ic_->commitString(output.commit);

            fcitx::Text empty("");
            ic_->inputPanel().setClientPreedit(empty);
            ic_->updatePreedit();
        }
        break;

    case VIME_ACTION_UPDATE_PREEDIT:
        if (output.rendered) {
            VIME_IF_DEV({
                VIME_DEBUG()
                    << "preedit: "
                    << output.rendered;
            });

            fcitx::Text text(output.rendered);

            ic_->inputPanel().setClientPreedit(text);
            ic_->updatePreedit();
        }
        break;

    case VIME_ACTION_NOOP:
    case VIME_ACTION_FORWARD:
    default:
        break;
    }
}

} // namespace vime::fcitx5
