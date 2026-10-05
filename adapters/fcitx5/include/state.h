#pragma once

#include "vime_engine.h"
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx-utils/key.h>


namespace vime::fcitx5 {

class Vime;

class VimeState final : public fcitx::InputContextProperty {
public:
  VimeState(Vime *engine, fcitx::InputContext *ic);
  ~VimeState() override;

  // Non-copyable, non-movable (managed by Fcitx5 InputContextManager)
  VimeState(const VimeState &) = delete;
  VimeState &operator=(const VimeState &) = delete;
  VimeState(VimeState &&) = delete;
  VimeState &operator=(VimeState &&) = delete;

  void keyEvent(fcitx::KeyEvent &event);
  void reset();

private:
  void showPreedit();
  void clearPreedit();
  void commitPreedit();

private:

  Vime *engine_{nullptr};
  fcitx::InputContext *ic_{nullptr};
  VimeSessionHandle *session_{nullptr};
};

}
