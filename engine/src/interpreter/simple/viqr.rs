use super::{InterpreterConfig, ShapeConfig, ShapeFamily, ToneConfig};
use crate::{RootVowel, Shape, Tone};

/// Standard VIQR keyboard mappings.
pub(crate) const CONFIG: &InterpreterConfig = &InterpreterConfig {
    tone_keys: &[
        ToneConfig {
            key: '\'',
            tone: Tone::Acute,
        },
        ToneConfig {
            key: '`',
            tone: Tone::Grave,
        },
        ToneConfig {
            key: '?',
            tone: Tone::Hook,
        },
        ToneConfig {
            key: '~',
            tone: Tone::Tilde,
        },
        ToneConfig {
            key: '.',
            tone: Tone::Dot,
        },
        ToneConfig {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    shape_keys: &[
        ShapeConfig {
            key: '^',
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '^',
            target: ShapeFamily::Vowel(RootVowel::E),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '^',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Circumflex,
        },
        ShapeConfig {
            key: '(',
            target: ShapeFamily::Vowel(RootVowel::A),
            shape: Shape::Breve,
        },
        ShapeConfig {
            key: '+',
            target: ShapeFamily::Vowel(RootVowel::O),
            shape: Shape::Horn,
        },
        ShapeConfig {
            key: '+',
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
