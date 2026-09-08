use super::super::{InterpreterConfig, ShapeConfig, ToneConfig};
use crate::{RootVowel, Shape, ShapeFamily, Tone};

pub(crate) const CONFIG: &InterpreterConfig = &InterpreterConfig {
    tone_keys: &[
        ToneConfig {
            key: 's',
            tone: Tone::Acute,
        },
        ToneConfig {
            key: 'f',
            tone: Tone::Grave,
        },
        ToneConfig {
            key: 'r',
            tone: Tone::Hook,
        },
        ToneConfig {
            key: 'x',
            tone: Tone::Tilde,
        },
        ToneConfig {
            key: 'j',
            tone: Tone::Dot,
        },
        ToneConfig {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    shape_keys: &[
        ShapeConfig {
            key: 'a',
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Breve,
        },
        ShapeConfig {
            key: 'e',
            target: ShapeFamily::Vowel(RootVowel::E),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'o',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeFamily::Vowel(RootVowel::U),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: 'd',
            target: ShapeFamily::D,
            shape: Shape::Stroke,
        },
    ],
};
