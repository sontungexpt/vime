use crate::phonology::{Shape, Tone};

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyContext {
    pub target: Option<char>,
}

impl KeyContext {
    pub fn new(target: Option<char>) -> Self {
        Self { target }
    }
}

/// Interprets keyboard input into semantic Vietnamese actions.
pub trait KeyInterpreter {
    fn is_transform_key(&self, input: char) -> bool;

    fn interpret_shape(&self, context: KeyContext, input: char) -> Option<(char, Shape)>;

    fn interpret_tone(&self, context: KeyContext, input: char) -> Option<(char, Tone)>;
}
