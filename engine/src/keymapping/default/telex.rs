use super::{InputLayout, ShapeMapping, ToneMapping};
use crate::{RootVowel, Shape, Tone};

/// Telex layout: shapes on `a/e/o` (circumflex), `w` (breve/horn), `d`
/// (stroke); tones on `s/f/r/x/j/z`.
pub(crate) const CONFIG: &InputLayout = &InputLayout::new(
    &[
        ToneMapping {
            key: 's',
            tone: Tone::Acute,
        },
        ToneMapping {
            key: 'f',
            tone: Tone::Grave,
        },
        ToneMapping {
            key: 'r',
            tone: Tone::Hook,
        },
        ToneMapping {
            key: 'x',
            tone: Tone::Tilde,
        },
        ToneMapping {
            key: 'j',
            tone: Tone::Dot,
        },
        ToneMapping {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMapping {
            key: 'a',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: 'w',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMapping {
            key: 'e',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: 'o',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: 'w',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMapping {
            key: 'w',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
