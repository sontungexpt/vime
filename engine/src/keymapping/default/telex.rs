use super::{KeyConfig, ShapeMap, ToneMap};
use crate::{RootVowel, Shape, Tone};

/// Telex layout: shapes on `a/e/o` (circumflex), `w` (breve/horn), `d`
/// (stroke); tones on `s/f/r/x/j/z`.
pub(crate) const CONFIG: &KeyConfig = &KeyConfig::new(
    &[
        ToneMap {
            key: 's',
            tone: Tone::Acute,
        },
        ToneMap {
            key: 'f',
            tone: Tone::Grave,
        },
        ToneMap {
            key: 'r',
            tone: Tone::Hook,
        },
        ToneMap {
            key: 'x',
            tone: Tone::Tilde,
        },
        ToneMap {
            key: 'j',
            tone: Tone::Dot,
        },
        ToneMap {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMap {
            key: 'a',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'w',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMap {
            key: 'e',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'o',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'w',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMap {
            key: 'w',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
