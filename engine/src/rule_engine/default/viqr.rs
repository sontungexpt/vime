use super::{ShapeRule, ToneRule, TypingRules};
use crate::{RootVowel, Shape, Tone};

/// VIQr layout: shapes on `^` (circumflex), `(` (breve), `+` (horn), `d`
/// (stroke); tones on `` ` `` `?` `~` `'` `.` and `z`.
pub(crate) const CONFIG: &TypingRules = &TypingRules::new(
    &[
        ToneRule {
            key: '`',
            tone: Tone::Grave,
        },
        ToneRule {
            key: '?',
            tone: Tone::Hook,
        },
        ToneRule {
            key: '~',
            tone: Tone::Tilde,
        },
        ToneRule {
            key: '\'',
            tone: Tone::Acute,
        },
        ToneRule {
            key: '.',
            tone: Tone::Dot,
        },
        ToneRule {
            key: 'z',
            tone: Tone::Flat,
        },
    ],
    &[
        ShapeRule {
            key: '^',
            vowel: RootVowel::A,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '^',
            vowel: RootVowel::E,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '^',
            vowel: RootVowel::O,
            shape: Shape::Circumflex,
        },
        ShapeRule {
            key: '(',
            vowel: RootVowel::A,
            shape: Shape::Breve,
        },
        ShapeRule {
            key: '+',
            vowel: RootVowel::O,
            shape: Shape::Horn,
        },
        ShapeRule {
            key: '+',
            vowel: RootVowel::U,
            shape: Shape::Horn,
        },
    ],
    &['d'],
);
