#pragma once

#include "vime_engine.h"
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx-utils/key.h>

namespace vime::fcitx5 {

class VimeEngine;

class VimeState final : public fcitx::InputContextProperty {
public:
  VimeState(VimeEngine *engine, fcitx::InputContext *ic);
  ~VimeState() override;

  // Non-copyable, non-movable (managed by Fcitx5 InputContextManager)
  VimeState(const VimeState &) = delete;
  VimeState &operator=(const VimeState &) = delete;
  VimeState(VimeState &&) = delete;
  VimeState &operator=(VimeState &&) = delete;

  void keyEvent(fcitx::KeyEvent &event);
  void reset();
  void setInputMethod(VimeInputMethod method);

  void apply(VimeOutput output);

private:
  VimeEngine *engine_{nullptr};
  fcitx::InputContext *ic_{nullptr};
  VimeEngineHandle *handle_{nullptr};
};

}
