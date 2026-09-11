use super::{InputLayout, ShapeMapping, ToneMapping};
use crate::{RootVowel, Shape, Tone};

/// VIQr layout: shapes on `^` (circumflex), `(` (breve), `+` (horn), `d`
/// (stroke); tones on `` ` `` `?` `~` `'` `.` and `z`.
pub(crate) const CONFIG: &InputLayout = &InputLayout::new(
    &[
        ToneMapping {
            key: '`',
            tone: Tone::Grave,
        },
        ToneMapping {
            key: '?',
            tone: Tone::Hook,
        },
        ToneMapping {
            key: '~',
            tone: Tone::Tilde,
        },
        ToneMapping {
            key: '\'',
            tone: Tone::Acute,
        },
        ToneMapping {
            key: '.',
            tone: Tone::Dot,
        },
        ToneMapping {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMapping {
            key: '^',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '^',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '^',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '(',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMapping {
            key: '+',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMapping {
            key: '+',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
