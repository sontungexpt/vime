use crate::phonology::decode_vowel;
use crate::{Shape, Tone};

mod config;
mod telex;
mod viqr;
mod vni;

pub use config::{InputLayout, ShapeMapping, ToneMapping};

use super::api::KeyTarget;
use super::KeyMapping;

/// Configuration-driven key mapping implementation.
///
/// This is used by input methods whose behavior can be described
/// declaratively through a [`InputLayout`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultKeyMapping<'a> {
    config: &'a InputLayout<'a>,
}

impl<'a> DefaultKeyMapping<'a> {
    /// Creates a key mapping from a declarative configuration.
    pub const fn new(config: &'a InputLayout<'a>) -> Self {
        Self { config }
    }

    /// The Telex input method.
    #[inline(always)]
    pub const fn telex() -> Self {
        Self::new(telex::CONFIG)
    }

    /// The VNI input method.
    #[inline(always)]
    pub const fn vni() -> Self {
        Self::new(vni::CONFIG)
    }

    /// The VIQR input method.
    #[inline(always)]
    pub const fn viqr() -> Self {
        Self::new(viqr::CONFIG)
    }

    /// The underlying configuration.
    #[inline(always)]
    pub const fn config(&self) -> &InputLayout<'a> {
        self.config
    }
}

impl KeyMapping for DefaultKeyMapping<'_> {
    /// Returns whether `key` is configured as a tone, shape, or stroke key.
    #[inline(always)]
    fn is_transform(&self, key: char) -> bool {
        let key = key.to_ascii_lowercase();

        self.config.tones.iter().any(|map| map.key == key)
            || self.config.strokes.iter().any(|&k| k == key)
            || self.config.shapes.iter().any(|map| map.key == key)
    }

    #[inline(always)]
    fn tone(&self, input: char) -> Option<Tone> {
        let loinput = input.to_ascii_lowercase();

        self.config
            .tones
            .iter()
            .find(|map| map.key == loinput)
            .map(|map| map.tone)
    }

    #[inline(always)]
    fn stroke(&self, input: char) -> bool {
        let loinput = input.to_ascii_lowercase();
        self.config.strokes.iter().any(|c| *c == loinput)
    }

    #[inline(always)]
    fn shape(&self, input: char, target: KeyTarget) -> Option<Shape> {
        let loinput = input.to_ascii_lowercase();

        // Only vowel families are shapeable. Resolve the target into a
        // `RootVowel` owner:
        //  - a char resolves through the codec (`ắ`/`Ắ` → `A`);
        //  - a decoded `BaseVowel` maps straight to its root.
        // Anything else — `d`/`đ`, `x`, `q`, `1`, arbitrary Unicode — matches
        // no shape key and passes through.
        let owner = match target {
            KeyTarget::Char(c) => {
                let Some((base, _, _)) = decode_vowel(c) else {
                    return None;
                };
                base.root()
            }
            KeyTarget::BaseVowel(base) => base.root(),
        };

        self.config
            .shapes
            .iter()
            .find(|map| map.key == loinput && map.vowel == owner)
            .map(|map| map.shape)
    }
}
