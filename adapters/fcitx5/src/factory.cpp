#include "engine.h"
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>

namespace vime::fcitx5 {

FCITX_DEFINE_LOG_CATEGORY(vime, "vime");

class VimeFactory final : public fcitx::AddonFactory {
public:
  fcitx::AddonInstance *create(fcitx::AddonManager *manager) override
  {
    return new VimeEngine(manager->instance());
  }
};

FCITX_ADDON_FACTORY(VimeFactory);

}
