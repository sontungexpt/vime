use crate::phonology::{BaseVowel, Shape, Tone};

/// What a shape key may be applied to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyTarget {
    /// A raw character in the buffer.
    Char(char),
    /// A parsed Vietnamese base vowel.
    BaseVowel(BaseVowel),
}

/// Interprets keyboard input into semantic Vietnamese actions.
pub trait KeyMapping {
    /// Returns whether `input` is configured as a tone, shape, or stroke key.
    fn is_transform(&self, input: char) -> bool;

    /// Interprets `input` as a tone key, returning the configured [`Tone`].
    fn tone(&self, input: char) -> Option<Tone>;

    /// Interprets `input` as the `d`/`đ` stroke key.
    fn stroke(&self, input: char) -> bool;

    /// Interprets `input` as a shape key for `target`, returning the
    /// configured [`Shape`].
    fn shape(&self, input: char, target: KeyTarget) -> Option<Shape>;
}
