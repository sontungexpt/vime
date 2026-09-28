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

    // A bare modifier press carries no character, but Ctrl+<letter> does: the
    // keysym is still a letter, so it would otherwise reach the engine and be
    // inserted instead of reaching the application. Every shortcut would become
    // stray text in the preedit. Forward anything with a modifier and let the
    // application have it.
    if (key.hasModifier()) {
        return;
    }

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
    if (!handle_ || !vime_reset(handle_)) {
        return;
    }

    // The buffer is empty, so the preedit must go with it. `vime_reset` has no
    // action to dispatch -- it consumes no key and commits nothing -- so the
    // repaint is this method's own responsibility rather than `apply`'s.
    ic_->inputPanel().setClientPreedit(fcitx::Text(""));
    ic_->updatePreedit();
}

void VimeState::setInputMethod(VimeInputMethod method)
{
    if (!handle_) {
        return;
    }

    // Switching the method clears the buffer, so on success the word has
    // changed and must be re-read. The C entry point reports only success, so
    // there is no action to dispatch here.
    if (vime_set_input_method(handle_, method)) {
        showPreedit();
    }
}

void VimeState::showPreedit()
{
    if (!ic_) {
        return;
    }

    // "Preedit" is fcitx's word for this; the engine just calls it the word it
    // parsed. Fetched only now, when something actually needs it — a caret
    // move leaves the text alone but still has to redraw, so both callers read
    // it the same way.
    const char *word = vime_parsed(handle_);

    if (!word) {
        return;
    }

    VIME_IF_DEV({
        VIME_DEBUG()
            << "preedit: "
            << word;
    });

    ic_->inputPanel().setClientPreedit(fcitx::Text(word));
    ic_->updatePreedit();
}

void VimeState::apply(VimeOutput output)
{
    if (!ic_) {
        return;
    }

    switch (output.action) {
    case VIME_ACTION_COMMIT: {
        // The commit text comes from the output that produced it, rather than
        // from a separate accessor: the field is set on VIME_ACTION_COMMIT and
        // NULL for every other action, so there is no window in which a stale
        // commit could be read.
        if (!output.commit) {
            break;
        }

        VIME_INFO()
            << "commit: "
            << output.commit;

        ic_->commitString(output.commit);

        // Committing clears the engine's buffer, so the preedit window goes
        // with it.
        ic_->inputPanel().setClientPreedit(fcitx::Text(""));
        ic_->updatePreedit();
        break;
    }

    case VIME_ACTION_CHANGED:
    case VIME_ACTION_CURSOR_MOVED:
        showPreedit();
        break;

    case VIME_ACTION_NOOP:
    case VIME_ACTION_FORWARD:
    default:
        break;
    }
}

} // namespace vime::fcitx5
