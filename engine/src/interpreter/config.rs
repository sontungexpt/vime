use crate::{RootVowel, Shape, Tone};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ShapeFamily {
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
    pub target: ShapeFamily,
    pub shape: Shape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterpreterConfig<'a> {
    pub tone_keys: &'a [ToneConfig],
    pub shape_keys: &'a [ShapeConfig],
}
