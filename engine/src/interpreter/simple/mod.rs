use crate::phonology::decode_vowel_base;
use crate::{Shape, Tone};

mod config;
mod telex;
mod viqr;
mod vni;

pub use config::{InterpreterConfig, ShapeConfig, ShapeFamily, ToneConfig};

use super::api::KeyContext;
use super::KeyInterpreter;

/// Configuration-driven interpreter implementation.
///
/// This is used by input methods whose behavior can be described
/// declaratively through [`InterpreterConfig`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimpleInterpreter<'a> {
    config: &'a InterpreterConfig<'a>,
}

impl<'a> SimpleInterpreter<'a> {
    pub const fn new(config: &'a InterpreterConfig<'a>) -> Self {
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
    pub const fn config(&self) -> &InterpreterConfig<'a> {
        self.config
    }
}

impl KeyInterpreter for SimpleInterpreter<'_> {
    /// Returns whether `key` is configured as a tone or shape key.
    #[inline(always)]
    fn is_transform_key(&self, key: char) -> bool {
        let key = key.to_ascii_lowercase();

        let mut i = 0;
        while i < self.config.tone_keys.len() {
            if self.config.tone_keys[i].key == key {
                return true;
            }
            i += 1;
        }

        let mut i = 0;
        while i < self.config.shape_keys.len() {
            if self.config.shape_keys[i].key == key {
                return true;
            }
            i += 1;
        }

        false
    }

    #[inline(always)]
    fn interpret_shape(&self, context: KeyContext, input: char) -> Option<(char, Shape)> {
        if let Some(target) = context.target {
            // Only vowel families and the special `d` family (`d`/`đ`) are
            // shapeable. Vowel families resolve through the codec (`ắ`/`Ắ`
            // → `A`); `d`/`D`/`đ`/`Đ` resolve to the `d` family. Anything
            // else — `x`, `q`, `1`, arbitrary Unicode — matches no shape key
            // and passes through.

            let shapeable = match target {
                'd' | 'D' | 'đ' | 'Đ' => Some(ShapeFamily::D),
                _ => decode_vowel_base(target)
                    .ok()
                    .map(|base| ShapeFamily::Vowel(base.root())),
            };

            let linput = input.to_ascii_lowercase();
            if let Some(target) = shapeable {
                if let Some(entry) = self
                    .config
                    .shape_keys
                    .iter()
                    .find(|entry| entry.target == target && entry.key == linput)
                {
                    return Some((input, entry.shape));
                }
            }
        }
        None
    }

    #[inline(always)]
    fn interpret_tone(&self, context: KeyContext, input: char) -> Option<(char, Tone)> {
        let config = self.config;
        let lk = input.to_ascii_lowercase();

        if let Some(entry) = config.tone_keys.iter().find(|entry| entry.key == lk) {
            return Some((input, entry.tone));
        }
        None
    }
}
