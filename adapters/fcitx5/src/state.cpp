#include "state.h"
#include "engine.h"
#include "log.h"

#include <fcitx-utils/keysym.h>
#include <fcitx-utils/log.h>
#include <fcitx/inputpanel.h>

#include <iomanip>
#include <string>

namespace vime::fcitx5 {

VimeState::VimeState(Vime *engine, fcitx::InputContext *ic)
    : engine_(engine),
      ic_(ic),
      session_(vime_session_create(engine_->sessionFactory()))
{
}

VimeState::~VimeState()
{
    if (session_) {
        vime_session_destroy(session_);
        session_ = nullptr;
    }
}

void VimeState::showPreedit()
{
    if (!ic_ || !session_) {
        return;
    }

    const VimeStringView rendered =
        vime_session_get_rendered(session_);

    fcitx::Text preedit;

    if (rendered.len != 0) {
        preedit.append(
            std::string(rendered.data, rendered.len),
            fcitx::TextFormatFlag::Underline);

        const size_t cursor =
            vime_session_get_rendered_cursor(session_);

        // fcitx::Text cursor is a UTF-8 byte position.
        preedit.setCursor(static_cast<int>(cursor));
        VIME_IF_DEV({
            VIME_DEBUG()
                << "showPreedit: rendered="
                << std::string(rendered.data, rendered.len)
                << " cursor="
                << cursor;
        });
    }

    const bool useClientPreedit =
        ic_->capabilityFlags().test(fcitx::CapabilityFlag::Preedit);

    if (useClientPreedit) {
        ic_->inputPanel().setClientPreedit(preedit);
    } else {
        ic_->inputPanel().setPreedit(preedit);
    }

    ic_->updatePreedit();
}

void VimeState::clearPreedit()
{
    if (!ic_) {
        return;
    }

    ic_->inputPanel().reset();
    ic_->updatePreedit();
}

void VimeState::commitPreedit()
{
    if (!ic_ || !session_) {
        return;
    }

    const VimeStringView rendered =
        vime_session_get_rendered(session_);

    if (rendered.len != 0) {
        ic_->commitString(
            std::string(rendered.data, rendered.len));
    }

    vime_session_reset(session_);
    clearPreedit();
}

void VimeState::keyEvent(fcitx::KeyEvent &event)
{
    if (event.isRelease() || !session_ || !ic_) {
        return;
    }

    const auto key = event.key();

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
        if (vime_session_backspace(session_)) {
            showPreedit();
            event.filterAndAccept();
        }
        return;

    case FcitxKey_Delete:
        if (vime_session_delete(session_)) {
            showPreedit();
            event.filterAndAccept();
        }
        return;

    case FcitxKey_Left:
        if (vime_session_move_cursor_left(session_, 1)) {
            showPreedit();
            event.filterAndAccept();
        }
        return;

    case FcitxKey_Right:
        if (vime_session_move_cursor_right(session_, 1)) {
            showPreedit();
            event.filterAndAccept();
        }
        return;

    case FcitxKey_Escape:
        /*
         * Escape cancels the current composition.
         *
         * Only consume Escape when there is actually something to cancel.
         */
        if (vime_session_get_rendered_len(session_) != 0) {
            vime_session_reset(session_);
            clearPreedit();
            event.filterAndAccept();
        }
        return;

    case FcitxKey_space:
        /*
         * Commit the composition. Do not filter Space so that the original
         * Space key can still reach the application.
         */
        if (vime_session_get_rendered_len(session_) != 0) {
            commitPreedit();
        }
        return;

    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
        /*
         * Commit the composition. Do not filter Enter so the application
         * receives its original Enter key.
         */
        if (vime_session_get_rendered_len(session_) != 0) {
            commitPreedit();
        }
        return;

    case FcitxKey_Tab:
        /*
         * Same policy as Space/Enter for now:
         * commit the composition and let Tab reach the application.
         */
        if (vime_session_get_rendered_len(session_) != 0) {
            commitPreedit();
        }
        return;

    default:
        break;
    }

    /*
     * Only process keys that produce a Unicode scalar.
     */
    const uint32_t character =
        fcitx::Key::keySymToUnicode(key.sym());

    if (character == 0) {
        return;
    }

    const VimeInsertResult result =
        vime_session_insert(session_, character);

    if (result.kind == VIME_INSERT_INVALID) {
        /*
         * The character is not accepted by the current composition.
         *
         * For the first implementation, commit the existing composition
         * and let the original key continue to the application.
         */
        if (vime_session_get_rendered_len(session_) != 0) {
            commitPreedit();
        }

        return;
    }

    showPreedit();
    event.filterAndAccept();

    VIME_IF_DEV({
        VIME_DEBUG()
            << "insert action="
            << static_cast<int>(result.kind)
            << " first_changed="
            << result.first_changed;
    });
}

void VimeState::reset()
{
    if (!session_ || !ic_) {
        return;
    }

    vime_session_reset(session_);
    clearPreedit();
}

} // namespace vime::fcitx5
