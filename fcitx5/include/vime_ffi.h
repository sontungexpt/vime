#pragma once
/* C ABI between the native Fcitx5 adapter and the Rust backend. */

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct VimeEngineHandle VimeEngineHandle;

typedef struct VimeOutput {
    bool consumed;   /* true if the IME consumed the key (call filterAndAccept) */
    bool changed;    /* true if rendered or commit differ from the previous call */
    char *rendered;  /* preedit text (UTF-8); NULL if unchanged; must be freed with vime_free_string */
    char *commit;    /* finished text to commit (UTF-8); NULL if no commit; must be freed with vime_free_string */
} VimeOutput;

typedef uint32_t VimeKey;
#define VIME_KEY_BACKSPACE         1u
#define VIME_KEY_DELETE            2u
#define VIME_KEY_LEFT              3u
#define VIME_KEY_RIGHT             4u
#define VIME_KEY_ENTER             5u
#define VIME_KEY_ESCAPE            6u
#define VIME_KEY_TAB               7u
#define VIME_KEY_SPACE             8u

/* Key event state matches fcitx5 KeyState bit positions exactly. */
#define VIME_KEY_STATE_SHIFT     (1u << 0)
#define VIME_KEY_STATE_CAPS_LOCK (1u << 1)
#define VIME_KEY_STATE_CTRL      (1u << 2)
#define VIME_KEY_STATE_ALT       (1u << 3)
#define VIME_KEY_STATE_NUM_LOCK  (1u << 4)
#define VIME_KEY_STATE_HYPER     (1u << 5)
#define VIME_KEY_STATE_SUPER     (1u << 6)
#define VIME_KEY_STATE_META      (1u << 28)

typedef struct VimeKeyEvent {
    VimeKey key;
    uint32_t character;
    uint32_t state;  /* fcitx5 KeyState bitmask — pass through directly */
} VimeKeyEvent;

typedef uint32_t VimeInputMethod;
#define VIME_INPUT_METHOD_TELEX 1u
#define VIME_INPUT_METHOD_VNI   2u

VimeEngineHandle *vime_create(void);
void vime_destroy(VimeEngineHandle *);
VimeOutput vime_reset(VimeEngineHandle *);

VimeOutput vime_process_key(VimeEngineHandle *, VimeKeyEvent event);

void vime_set_method(VimeEngineHandle *, VimeInputMethod method);
void vime_free_string(char *);

#ifdef __cplusplus
}
#endif
