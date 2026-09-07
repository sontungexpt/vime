use super::{InterpreterConfig, ShapeConfig, ToneConfig};
use crate::{RootVowel, Shape, ShapeTarget, Tone};

pub(crate) const CONFIG: &InterpreterConfig = &InterpreterConfig {
    tone_keys: &[
        ToneConfig {
            key: '1',
            tone: Tone::Acute,
        },
        ToneConfig {
            key: '2',
            tone: Tone::Grave,
        },
        ToneConfig {
            key: '3',
            tone: Tone::Hook,
        },
        ToneConfig {
            key: '4',
            tone: Tone::Tilde,
        },
        ToneConfig {
            key: '5',
            tone: Tone::Dot,
        },
        ToneConfig {
            key: '0',
            tone: Tone::Flat,
        },
    ],
    shape_keys: &[
        ShapeConfig {
            key: '6',
            target: ShapeTarget::Vowel(RootVowel::A),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '7',
            target: ShapeTarget::Vowel(RootVowel::A),
            shape: Shape::Breve,
        },
        ShapeConfig {
            key: '6',
            target: ShapeTarget::Vowel(RootVowel::E),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '6',
            target: ShapeTarget::Vowel(RootVowel::O),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '7',
            target: ShapeTarget::Vowel(RootVowel::O),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: '8',
            target: ShapeTarget::Vowel(RootVowel::U),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: '9',
            target: ShapeTarget::D,
            shape: Shape::Stroke,
        },
    ],
};
