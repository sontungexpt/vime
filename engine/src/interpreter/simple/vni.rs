use super::super::{InterpreterConfig, ShapeConfig, ToneConfig};
use crate::{RootVowel, Shape, ShapeFamily, Tone};

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
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '7',
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Breve,
        },
        ShapeConfig {
            key: '6',
            target: ShapeFamily::Vowel(RootVowel::E),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '6',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '7',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: '8',
            target: ShapeFamily::Vowel(RootVowel::U),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: '9',
            target: ShapeFamily::D,
            shape: Shape::Stroke,
        },
    ],
};
