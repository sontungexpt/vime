use crate::phonology::{Shape, Tone};

/// Semantic Vietnamese text operation produced by interpreting a keystroke.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    // char is the input character that triggered this operation
    Insert(char),
    // char is the input character that triggered this operation
    Shapeable(char, Shape),
    // char is the input character that triggered this operation
    Toneable(char, Tone),
}
