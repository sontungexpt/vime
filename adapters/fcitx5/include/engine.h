#pragma once

#include "state.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <vector>

namespace vime::fcitx5 {

class VimeEngine final : public fcitx::InputMethodEngine {
public:
  explicit VimeEngine(fcitx::Instance *instance);
  ~VimeEngine() override = default;

  std::vector<fcitx::InputMethodEntry> listInputMethods() override;
  void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &event) override;

  fcitx::Instance *instance() const { return instance_; }

private:
  fcitx::Instance *instance_{nullptr};
  fcitx::FactoryFor<VimeState> factory_;
};

}
