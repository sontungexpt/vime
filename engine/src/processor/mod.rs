//! Vietnamese syllable rules and semantic text processing.
//!
//! Two entry points serve different layers:
//!
//! - [`Processor`] applies semantic [`Operation`]s to a semantic [`Vowel`],
//!   independent of keyboard layout and raw composition.
//! - the analysis functions (`[`parse`](crate::processor::parse)`,
//!   `[`analyze_syllable`]`, `[`normalize`]`, `[`render_word`]`) work over
//!   canonical ASCII raw syllables and compose them into Vietnamese text.

mod normalize;
mod render;
mod rules;
mod syllable;
mod tone;
mod vowel;

use crate::Operation;
use crate::{character::Vowel};

pub use normalize::{canonicalize_into, normalize, render_raw};
pub use render::render_word;
pub use rules::{Coda, Onset};
pub use syllable::{
    analyze_syllable, analyze_syllable_with_orthography, parse, SequenceState, SyllableAnalysis,
    WordStructure,
};
pub use tone::Orthography;

pub(crate) use syllable::serialize;

/// Semantic Vietnamese text processor.
///
/// Transforms a [`Vowel`] by applying the semantic portion of an
/// [`Operation`]. Shape and tone operations update the semantic model; other
/// operations (insertions, deletions, consonant strokes) do not apply and
/// yield `None`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Processor;

impl Processor {
    pub const fn new() -> Self {
        Self
    }

    /// Applies an [`Operation`] to the semantic vowel `target`.
    pub fn apply(&self, target: Vowel, operation: Operation) -> Option<Vowel> {
        match operation {
            Operation::Shapeable(_, shape) => target.with_shape(shape),
            Operation::Toneable(_, tone) => Some(target.with_tone(tone)),
            Operation::Insert(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
