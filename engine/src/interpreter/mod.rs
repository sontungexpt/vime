use crate::character::codec::decode_vowel_base;

#[cfg(test)]
use crate::Shape;

mod config;
mod operation;
mod telex;
mod vni;

pub use config::{InterpreterConfig, ShapeConfig, ShapeTarget, ToneConfig};
pub use operation::Operation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyContext {
    pub target: Option<char>,
}

/// Interprets keyboard input into semantic Vietnamese operations.
pub trait Interpreter {
    fn interpret(&self, context: KeyContext, key: char) -> Operation;
}

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

    pub const fn telex() -> Self {
        Self::new(telex::CONFIG)
    }

    pub const fn vni() -> Self {
        Self::new(vni::CONFIG)
    }

    pub const fn config(&self) -> &InterpreterConfig<'a> {
        self.config
    }
}

impl Interpreter for SimpleInterpreter<'_> {
    fn interpret(&self, context: KeyContext, key: char) -> Operation {
        let config = self.config;
        let lk = key.to_ascii_lowercase();

        if let Some(entry) = config.tone_keys.iter().find(|entry| entry.key == lk) {
            return Operation::Toneable(key, entry.tone);
        }

        if let Some(target) = context.target {
            // Only vowel families and the special `d` family (`d`/`đ`) are
            // shapeable. Vowel families resolve through the codec (`ắ`/`Ắ`
            // → `A`); `d`/`D`/`đ`/`Đ` resolve to the `d` family. Anything
            // else — `x`, `q`, `1`, arbitrary Unicode — matches no shape key
            // and passes through.
            let shapeable = decode_vowel_base(target)
                .map(|base| ShapeTarget::Vowel(base.root()))
                .or(match target {
                    'd' | 'D' | 'đ' | 'Đ' => Some(ShapeTarget::D),
                    _ => None,
                });
            if let Some(target) = shapeable {
                if let Some(entry) = config
                    .shape_keys
                    .iter()
                    .find(|entry| entry.target == target && entry.key == lk)
                {
                    return Operation::Shapeable(key, entry.shape);
                }
            }
        }

        Operation::Insert(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tone;

    fn ctx(target: Option<char>) -> KeyContext {
        KeyContext { target }
    }

    #[test]
    fn telex_insert() {
        let p = SimpleInterpreter::telex();
        assert_eq!(p.interpret(ctx(None), 'a'), Operation::Insert('a'));
        assert_eq!(p.interpret(ctx(None), 'q'), Operation::Insert('q'));
        assert_eq!(p.interpret(ctx(Some('x')), 'a'), Operation::Insert('a'));
    }

    #[test]
    fn telex_shapes() {
        let p = SimpleInterpreter::telex();
        assert_eq!(
            p.interpret(ctx(Some('a')), 'a'),
            Operation::Shapeable('a', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('e')), 'e'),
            Operation::Shapeable('e', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('o')), 'o'),
            Operation::Shapeable('o', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'w'),
            Operation::Shapeable('w', Shape::Breve)
        );
        assert_eq!(
            p.interpret(ctx(Some('o')), 'w'),
            Operation::Shapeable('w', Shape::Horn)
        );
        assert_eq!(
            p.interpret(ctx(Some('u')), 'w'),
            Operation::Shapeable('w', Shape::Horn)
        );
        assert_eq!(
            p.interpret(ctx(Some('d')), 'd'),
            Operation::Shapeable('d', Shape::Stroke)
        );
    }

    #[test]
    fn shapes_apply_to_semantic_families() {
        let p = SimpleInterpreter::telex();
        assert_eq!(
            p.interpret(ctx(Some('ă')), 'w'),
            Operation::Shapeable('w', Shape::Breve)
        );
        assert_eq!(
            p.interpret(ctx(Some('ắ')), 'w'),
            Operation::Shapeable('w', Shape::Breve)
        );
        assert_eq!(
            p.interpret(ctx(Some('Ắ')), 'w'),
            Operation::Shapeable('w', Shape::Breve)
        );
        assert_eq!(
            p.interpret(ctx(Some('ấ')), 'a'),
            Operation::Shapeable('a', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('ớ')), 'w'),
            Operation::Shapeable('w', Shape::Horn)
        );
    }

    #[test]
    fn d_family_includes_precomposed_d_take() {
        let p = SimpleInterpreter::telex();
        assert_eq!(
            p.interpret(ctx(Some('đ')), 'd'),
            Operation::Shapeable('d', Shape::Stroke)
        );
        assert_eq!(
            p.interpret(ctx(Some('Đ')), 'd'),
            Operation::Shapeable('d', Shape::Stroke)
        );
        let p = SimpleInterpreter::vni();
        assert_eq!(
            p.interpret(ctx(Some('đ')), '9'),
            Operation::Shapeable('9', Shape::Stroke)
        );
    }

    #[test]
    fn non_shapeable_targets_are_not_shaped() {
        let p = SimpleInterpreter::telex();
        assert_eq!(p.interpret(ctx(Some('x')), 'd'), Operation::Insert('d'));
        assert_eq!(p.interpret(ctx(Some('1')), 'd'), Operation::Insert('d'));
        assert_eq!(p.interpret(ctx(Some('q')), 'w'), Operation::Insert('w'));
        assert_eq!(p.interpret(ctx(Some('Z')), 'w'), Operation::Insert('w'));
    }

    #[test]
    fn telex_tones() {
        let p = SimpleInterpreter::telex();
        // Tone keys map unconditionally; the raw buffer layer decides whether
        // a marker resolves into a real tone.
        assert_eq!(
            p.interpret(ctx(None), 's'),
            Operation::Toneable('s', Tone::Acute)
        );
        assert_eq!(
            p.interpret(ctx(None), 'f'),
            Operation::Toneable('f', Tone::Grave)
        );
        assert_eq!(
            p.interpret(ctx(None), 'r'),
            Operation::Toneable('r', Tone::Hook)
        );
        assert_eq!(
            p.interpret(ctx(None), 'x'),
            Operation::Toneable('x', Tone::Tilde)
        );
        assert_eq!(
            p.interpret(ctx(None), 'j'),
            Operation::Toneable('j', Tone::Dot)
        );
        assert_eq!(
            p.interpret(ctx(Some('x')), 's'),
            Operation::Toneable('s', Tone::Acute)
        );
        // With a vowel target, tone keys produce Tone.
        assert_eq!(
            p.interpret(ctx(Some('a')), 's'),
            Operation::Toneable('s', Tone::Acute)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'f'),
            Operation::Toneable('f', Tone::Grave)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'r'),
            Operation::Toneable('r', Tone::Hook)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'x'),
            Operation::Toneable('x', Tone::Tilde)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'j'),
            Operation::Toneable('j', Tone::Dot)
        );
        // A shape marker (`w`) standing for a breve/horn vowel still hosts tone.
        assert_eq!(
            p.interpret(ctx(Some('w')), 's'),
            Operation::Toneable('s', Tone::Acute)
        );
    }

    #[test]
    fn telex_remove_tone() {
        let p = SimpleInterpreter::telex();
        // `z` maps to Flat unconditionally; the raw buffer layer decides
        // whether a marker resolves into a real tone removal.
        assert_eq!(
            p.interpret(ctx(None), 'z'),
            Operation::Toneable('z', Tone::Flat)
        );
        assert_eq!(
            p.interpret(ctx(Some('x')), 'z'),
            Operation::Toneable('z', Tone::Flat)
        );
        // With a vowel target, z removes the tone.
        assert_eq!(
            p.interpret(ctx(Some('á')), 'z'),
            Operation::Toneable('z', Tone::Flat)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'z'),
            Operation::Toneable('z', Tone::Flat)
        );
        assert_eq!(
            p.interpret(ctx(Some('w')), 'z'),
            Operation::Toneable('z', Tone::Flat)
        );
    }

    #[test]
    fn telex_uppercase_lowercased() {
        let p = SimpleInterpreter::telex();
        // Keys pass through with their original case.
        assert_eq!(p.interpret(ctx(None), 'A'), Operation::Insert('A'));
        assert_eq!(
            p.interpret(ctx(None), 'S'),
            Operation::Toneable('S', Tone::Acute)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), 'S'),
            Operation::Toneable('S', Tone::Acute)
        );
    }

    #[test]
    fn vni_shapes() {
        let p = SimpleInterpreter::vni();
        assert_eq!(
            p.interpret(ctx(Some('a')), '6'),
            Operation::Shapeable('6', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), '7'),
            Operation::Shapeable('7', Shape::Breve)
        );
        assert_eq!(
            p.interpret(ctx(Some('e')), '6'),
            Operation::Shapeable('6', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('o')), '6'),
            Operation::Shapeable('6', Shape::Circumflex)
        );
        assert_eq!(
            p.interpret(ctx(Some('o')), '7'),
            Operation::Shapeable('7', Shape::Horn)
        );
        assert_eq!(
            p.interpret(ctx(Some('u')), '8'),
            Operation::Shapeable('8', Shape::Horn)
        );
        assert_eq!(
            p.interpret(ctx(Some('d')), '9'),
            Operation::Shapeable('9', Shape::Stroke)
        );
    }

    #[test]
    fn vni_tones() {
        let p = SimpleInterpreter::vni();
        // Tone keys map unconditionally; the raw buffer layer decides whether
        // a marker resolves into a real tone.
        assert_eq!(
            p.interpret(ctx(None), '1'),
            Operation::Toneable('1', Tone::Acute)
        );
        assert_eq!(
            p.interpret(ctx(None), '2'),
            Operation::Toneable('2', Tone::Grave)
        );
        assert_eq!(
            p.interpret(ctx(None), '3'),
            Operation::Toneable('3', Tone::Hook)
        );
        assert_eq!(
            p.interpret(ctx(None), '4'),
            Operation::Toneable('4', Tone::Tilde)
        );
        assert_eq!(
            p.interpret(ctx(None), '5'),
            Operation::Toneable('5', Tone::Dot)
        );
        // With a vowel target, tone keys produce Tone.
        assert_eq!(
            p.interpret(ctx(Some('a')), '1'),
            Operation::Toneable('1', Tone::Acute)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), '2'),
            Operation::Toneable('2', Tone::Grave)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), '3'),
            Operation::Toneable('3', Tone::Hook)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), '4'),
            Operation::Toneable('4', Tone::Tilde)
        );
        assert_eq!(
            p.interpret(ctx(Some('a')), '5'),
            Operation::Toneable('5', Tone::Dot)
        );
        // A shape marker (`w`) standing for a breve/horn vowel still hosts tone.
        assert_eq!(
            p.interpret(ctx(Some('w')), '1'),
            Operation::Toneable('1', Tone::Acute)
        );
    }

    #[test]
    fn custom_config() {
        let config = InterpreterConfig {
            tone_keys: &[ToneConfig {
                key: 'q',
                tone: Tone::Acute,
            }],
            shape_keys: &[],
        };
        let p = SimpleInterpreter::new(&config);
        assert_eq!(
            p.interpret(ctx(None), 'q'),
            Operation::Toneable('q', Tone::Acute)
        );
        assert_eq!(p.interpret(ctx(None), 'a'), Operation::Insert('a'));
        assert_eq!(
            p.interpret(ctx(Some('a')), 'q'),
            Operation::Toneable('q', Tone::Acute)
        );
    }
}
