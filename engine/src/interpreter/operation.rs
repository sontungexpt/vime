use crate::character::{Shape, Tone};

/// Semantic Vietnamese text operation produced by interpreting a keystroke.
///
/// Both [`Shapeable`](Operation::Shapeable) and
/// [`Toneable`](Operation::Toneable) carry the key that produced them; the
/// composition layer translates the shape/tone into canonical ASCII spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Insert(char),
    Shapeable(char, Shape),
    Toneable(char, Tone),
}