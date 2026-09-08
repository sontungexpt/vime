use crate::phonology::decode_vowel_base;

mod config;
mod telex;
mod vni;
mod viqr;

pub use config::{InterpreterConfig, ShapeConfig, ShapeFamily, ToneConfig};

use super::{api::KeyContext, Action};
use super::Interpreter;

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

    /// Returns whether `key` is configured as a tone or shape key.
    #[inline(always)]
    pub const fn is_modifier_key(&self, key: char) -> bool {
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
}

impl Interpreter for SimpleInterpreter<'_> {
    fn interpret(&self, context: KeyContext, input: char) -> Action {
        let config = self.config;
        let lk = input.to_ascii_lowercase();

        if let Some(entry) = config.tone_keys.iter().find(|entry| entry.key == lk) {
            return Action::Toneable(input, entry.tone);
        }

        if let Some(target) = context.target {
            // Only vowel families and the special `d` family (`d`/`đ`) are
            // shapeable. Vowel families resolve through the codec (`ắ`/`Ắ`
            // → `A`); `d`/`D`/`đ`/`Đ` resolve to the `d` family. Anything
            // else — `x`, `q`, `1`, arbitrary Unicode — matches no shape key
            // and passes through.
            let shapeable = decode_vowel_base(target)
                .map(|base| ShapeFamily::Vowel(base.root()))
                .or(match target {
                    'd' | 'D' | 'đ' | 'Đ' => Some(ShapeFamily::D),
                    _ => None,
                });
            if let Some(target) = shapeable {
                if let Some(entry) = config
                    .shape_keys
                    .iter()
                    .find(|entry| entry.target == target && entry.key == lk)
                {
                    return Action::Shapeable(input, entry.shape);
                }
            }
        }

        Action::Insert(input)
    }
}
