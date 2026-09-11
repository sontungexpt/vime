use super::{InputLayout, ShapeMapping, ToneMapping};
use crate::{RootVowel, Shape, Tone};

/// VNI layout: shapes on `6` (circumflex), `7` (breve/horn), `8` (horn),
/// `9` (stroke); tones on `1-5` and `0`.
pub(crate) const CONFIG: &InputLayout = &InputLayout::new(
    &[
        ToneMapping {
            key: '1',
            tone: Tone::Acute,
        },
        ToneMapping {
            key: '2',
            tone: Tone::Grave,
        },
        ToneMapping {
            key: '3',
            tone: Tone::Hook,
        },
        ToneMapping {
            key: '4',
            tone: Tone::Tilde,
        },
        ToneMapping {
            key: '5',
            tone: Tone::Dot,
        },
        ToneMapping {
            key: '0',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMapping {
            key: '6',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '7',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMapping {
            key: '6',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '6',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMapping {
            key: '7',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMapping {
            key: '8',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['9'],
);
