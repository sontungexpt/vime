use super::{KeyConfig, ShapeMap, ToneMap};
use crate::{RootVowel, Shape, Tone};

/// VNI layout: shapes on `6` (circumflex), `7` (breve/horn), `8` (horn),
/// `9` (stroke); tones on `1-5` and `0`.
pub(crate) const CONFIG: &KeyConfig = &KeyConfig::new(
    &[
        ToneMap {
            key: '1',
            tone: Tone::Acute,
        },
        ToneMap {
            key: '2',
            tone: Tone::Grave,
        },
        ToneMap {
            key: '3',
            tone: Tone::Hook,
        },
        ToneMap {
            key: '4',
            tone: Tone::Tilde,
        },
        ToneMap {
            key: '5',
            tone: Tone::Dot,
        },
        ToneMap {
            key: '0',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeMap {
            key: '6',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '7',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeMap {
            key: '6',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '6',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeMap {
            key: '7',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeMap {
            key: '8',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['9'],
);
