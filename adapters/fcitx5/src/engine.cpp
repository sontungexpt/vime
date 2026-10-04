#include "engine.h"
#include "log.h"

namespace vime::fcitx5 {

Vime::Vime(fcitx::Instance *instance)
  : instance_(instance)
  , stateFactory_([this](fcitx::InputContext &ic) {
      return new VimeState(this, &ic);
    })
  , sessionFactory_(vime_session_factory_create())
{
  instance_->inputContextManager().registerProperty(
      "vimeState",
      &stateFactory_);
}

Vime::~Vime()
{
    if (sessionFactory_) {
        vime_session_factory_destroy(sessionFactory_);
        sessionFactory_ = nullptr;
    }
}

std::vector<fcitx::InputMethodEntry> Vime::listInputMethods()
{
  std::vector<fcitx::InputMethodEntry> result;

  auto entry = fcitx::InputMethodEntry("vime", "Vime", "vi", "vime");
  entry.setIcon("input-keyboard").setLabel("vi");

  result.emplace_back(std::move(entry));
  return result;
}

void Vime::activate(
    const fcitx::InputMethodEntry &,
    fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();
  auto *state = ic->propertyFor(&stateFactory_);

  VIME_DEBUG() << "activated context " << ic
               << " state " << state;
}

void Vime::deactivate(
    const fcitx::InputMethodEntry &,
    fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();

  if (auto *state = ic->propertyFor(&stateFactory_)) {
    state->reset();
  }
}

void Vime::reset(
    const fcitx::InputMethodEntry &,
    fcitx::InputContextEvent &event)
{
  auto *ic = event.inputContext();

  if (auto *state = ic->propertyFor(&stateFactory_)) {
    state->reset();
  }
}

void Vime::keyEvent(
    const fcitx::InputMethodEntry &,
    fcitx::KeyEvent &event)
{
  auto *ic = event.inputContext();

  if (auto *state = ic->propertyFor(&stateFactory_)) {
    state->keyEvent(event);
  }
}

} // namespace vime::fcitx5
