#pragma once
/* C ABI between the native Fcitx5 adapter and the Rust backend. */

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct VietnameseFcitx5Engine VietnameseFcitx5Engine;

typedef struct VietnameseFcitx5Output {
    bool consumed;
    bool changed;
    char *rendered;
    char *commit;
} VietnameseFcitx5Output;

typedef enum VietnameseKey {
    VIETNAMESE_KEY_BACKSPACE = 1,
    VIETNAMESE_KEY_DELETE,
    VIETNAMESE_KEY_LEFT,
    VIETNAMESE_KEY_RIGHT,
    VIETNAMESE_KEY_HOME,
    VIETNAMESE_KEY_END,
    VIETNAMESE_KEY_ENTER,
    VIETNAMESE_KEY_ESCAPE,
    VIETNAMESE_KEY_TAB,
    VIETNAMESE_KEY_SPACE,
} VietnameseKey;

VietnameseFcitx5Engine *vietnamese_fcitx5_create(void);
void vietnamese_fcitx5_destroy(VietnameseFcitx5Engine *);
VietnameseFcitx5Output vietnamese_fcitx5_reset(VietnameseFcitx5Engine *);
VietnameseFcitx5Output vietnamese_fcitx5_process_character(
    VietnameseFcitx5Engine *, uint32_t character);
VietnameseFcitx5Output vietnamese_fcitx5_process_key(
    VietnameseFcitx5Engine *, VietnameseKey key);
void vietnamese_fcitx5_set_method(VietnameseFcitx5Engine *, bool vni);
void vietnamese_fcitx5_free_string(char *);

#ifdef __cplusplus
}
#endif
