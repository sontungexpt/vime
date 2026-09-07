use crate::{RootVowel, Shape, Tone};

/// A shapeable target: a vowel family or the special consonant `d`.
///
/// `d`/`D`/`đ`/`Đ` all map to [`ShapeTarget::D`]; vowel families resolve
/// through [`BaseVowel`]. Anything else is not shapeable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeTarget {
    Vowel(RootVowel),
    D,
}

/// A keyboard key mapped to a Vietnamese tone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToneConfig {
    pub key: char,
    pub tone: Tone,
}

/// A keyboard key mapped to a Vietnamese vowel shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapeConfig {
    pub key: char,
    pub target: ShapeTarget,
    pub shape: Shape,
}

/// Declarative configuration for one input method.
///
/// Tone keys are `(key, tone)` pairs; a `tone` of [`Tone::Flat`] marks a
/// "tone-removal" key (e.g. `z` in telex), since flat means no tone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterpreterConfig<'a> {
    pub tone_keys: &'a [ToneConfig],
    pub shape_keys: &'a [ShapeConfig],
}
