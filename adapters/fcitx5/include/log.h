#pragma once

#include <fcitx-utils/log.h>

FCITX_DECLARE_LOG_CATEGORY(vimeLog);

#define VIME_ERROR() FCITX_LOGC(vimeLog, Error)
#define VIME_WARN() FCITX_LOGC(vimeLog, Warn)
#define VIME_INFO() FCITX_LOGC(vimeLog, Info)
#define VIME_DEBUG() FCITX_LOGC(vimeLog, Debug)

#if defined(NDEBUG)
#define VIME_IF_DEV(...) \
    do {                 \
    } while (false)
#else
#define VIME_IF_DEV(...) \
    do {                 \
        __VA_ARGS__;     \
    } while (false)
#endif
