use crate::phonology::{Shape, Tone};

/// Semantic Vietnamese input action produced by interpreting a keystroke.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    // char is the input character that triggered this action
    Insert(char),
    // char is the input character that triggered this action
    Shapeable(char, Shape),
    // char is the input character that triggered this action
    Toneable(char, Tone),
}
