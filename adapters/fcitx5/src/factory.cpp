#include "engine.h"
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>

FCITX_DEFINE_LOG_CATEGORY(vimeLog, "vime");

namespace vime::fcitx5 {

class VimeFactory final : public fcitx::AddonFactory {
public:
  fcitx::AddonInstance *create(fcitx::AddonManager *manager) override
  {
    return new VimeEngine(manager->instance());
  }
};

FCITX_ADDON_FACTORY(VimeFactory);

}
