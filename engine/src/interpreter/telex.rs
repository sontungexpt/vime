use super::{InterpreterConfig, ShapeConfig, ToneConfig};
use crate::{RootVowel, Shape, ShapeTarget, Tone};

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
            target: ShapeTarget::Vowel(RootVowel::A),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeTarget::Vowel(RootVowel::A),
            shape: Shape::Breve,
        },
        ShapeConfig {
            key: 'e',
            target: ShapeTarget::Vowel(RootVowel::E),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'o',
            target: ShapeTarget::Vowel(RootVowel::O),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeTarget::Vowel(RootVowel::O),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: 'w',
            target: ShapeTarget::Vowel(RootVowel::U),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: 'd',
            target: ShapeTarget::D,
            shape: Shape::Stroke,
        },
    ],
};
