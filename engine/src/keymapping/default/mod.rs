use crate::phonology::decode_vowel_base;
use crate::{Shape, Tone};

mod config;
mod telex;
mod viqr;
mod vni;

pub use config::{KeyConfig, ShapeMap, ToneMap};

use super::api::KeyTarget;
use super::KeyMapping;

/// Configuration-driven interpreter implementation.
///
/// This is used by input methods whose behavior can be described
/// declaratively through [`InterpreterConfig`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefaultKeyMapping<'a> {
    config: &'a KeyConfig<'a>,
}

impl<'a> DefaultKeyMapping<'a> {
    pub const fn new(config: &'a KeyConfig<'a>) -> Self {
        Self { config }
    }

    #[inline(always)]
    pub const fn telex() -> Self {
        Self::new(telex::CONFIG)
    }

    #[inline(always)]
    pub const fn vni() -> Self {
        Self::new(vni::CONFIG)
    }

    #[inline(always)]
    pub const fn viqr() -> Self {
        Self::new(viqr::CONFIG)
    }

    #[inline(always)]
    pub const fn config(&self) -> &KeyConfig<'a> {
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
                let Some(base) = decode_vowel_base(c) else {
                    return None;
                };
                base.root()
            }
            KeyTarget::BaseVowel(base) => base.root(),
        };

        self.config
            .shapes
            .iter()
            .find(|map| map.key == loinput && map.owner == owner)
            .map(|map| map.shape)
    }
}
