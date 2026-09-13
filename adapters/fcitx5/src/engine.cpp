#include "engine.h"
#include "log.h"

namespace vime::fcitx5 {

VimeEngine::VimeEngine(fcitx::Instance *instance)
  : instance_(instance)
  , factory_([this](fcitx::InputContext &ic) { return new VimeState(this, &ic); })
{
  instance_->inputContextManager().registerProperty("vimeState", &factory_);
}

std::vector<fcitx::InputMethodEntry> VimeEngine::listInputMethods()
{
  std::vector<fcitx::InputMethodEntry> result;
  auto entry = fcitx::InputMethodEntry("vime", "Vime", "vi", "vime");
  entry.setIcon("input-keyboard").setLabel("vi");
  result.emplace_back(std::move(entry));
  return result;
}

void VimeEngine::activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();
  auto *state = ic->propertyFor(&factory_);
  VIME_DEBUG() << "activated context " << ic << " state " << state;
}

void VimeEngine::deactivate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();
  if (auto *state = ic->propertyFor(&factory_)) {
    state->reset();
  }
}

void VimeEngine::reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();
  if (auto *state = ic->propertyFor(&factory_)) {
    state->reset();
  }
}

void VimeEngine::keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event)
{
  auto *ic = event.inputContext();
  if (auto *state = ic->propertyFor(&factory_)) {
    state->keyEvent(event);
  }
}

}
