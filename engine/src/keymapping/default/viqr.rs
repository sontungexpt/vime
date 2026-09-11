use super::{KeyConfig, ShapeMap, ToneMap};
use crate::{RootVowel, Shape, Tone};

/// VIQr layout: shapes on `^` (circumflex), `(` (breve), `+` (horn), `d`
/// (stroke); tones on `` ` `` `?` `~` `'` `.` and `z`.
pub(crate) const CONFIG: &KeyConfig = &KeyConfig::new(
    &[
        ToneMap {
            key: '`',
            tone: Tone::Grave,
        },
        ToneMap {
            key: '?',
            tone: Tone::Hook,
        },
        ToneMap {
            key: '~',
            tone: Tone::Tilde,
        },
        ToneMap {
            key: '\'',
            tone: Tone::Acute,
        },
        ToneMap {
            key: '.',
            tone: Tone::Dot,
        },
        ToneMap {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMap {
            key: '^',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '^',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '^',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '(',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMap {
            key: '+',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMap {
            key: '+',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
