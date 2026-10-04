#pragma once

#include "state.h"
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <vector>

namespace vime::fcitx5 {

class Vime final : public fcitx::InputMethodEngine {
public:
  explicit Vime(fcitx::Instance *instance);
  ~Vime() override;

  std::vector<fcitx::InputMethodEntry> listInputMethods() override;
  void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
  void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &event) override;

  // Getters
  fcitx::Instance *instance() const { return instance_; }
  VimeSessionFactoryHandle *sessionFactory() const { return sessionFactory_; }

private:
  fcitx::Instance *instance_{nullptr};
  fcitx::FactoryFor<VimeState> stateFactory_;

  VimeSessionFactoryHandle *sessionFactory_{nullptr};
};

} // namespace vime::fcitx5
