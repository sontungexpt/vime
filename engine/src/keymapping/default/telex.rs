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
            owner: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'w',
            owner: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMap {
            key: 'e',
            owner: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'o',
            owner: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: 'w',
            owner: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMap {
            key: 'w',
            owner: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
