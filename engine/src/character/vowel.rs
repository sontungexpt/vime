//! Semantic Vietnamese vowel model.
//!
//! A Vietnamese vowel character (`a`, `ă`, `ắ`, `â`, `ấ`, ...) is represented
//! as the semantic triple `(BaseVowel, Tone, Case)`. The [`BaseVowel`] already
//! bundles the root letter and its shape (e.g. `a` + Breve = `ABreve`), so the
//! shape is not a separate field:
//!
//! ```text
//! a   → A                    + None  + Lower
//! ă   → ABreve (A + Breve)   + None  + Lower
//! á   → A                    + Acute + Lower
//! ắ   → ABreve (A + Breve)   + Acute + Lower
//! ấ   → ACircumflex (A + ^)  + Acute + Lower
//! Ạ   → A                    + Dot   + Upper
//! ```
//!
//! The internal representation is an implementation detail; callers use the
//! semantic accessors and mutation helpers below.

use super::{BaseVowel, Case, RootVowel, Shape, Tone};

/// A semantic Vietnamese vowel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Vowel {
    base: BaseVowel,
    tone: Tone,
    case: Case,
}

impl Vowel {
    /// Builds a vowel from its semantic parts.
    ///
    /// Infallible: every [`BaseVowel`] is a valid vowel, and tone/case are
    /// unrestricted. Consonant shapes (`Shape::Stroke`) cannot be represented
    /// because they live in [`BaseVowel`] (which has no Stroke form).
    pub const fn new(base: BaseVowel, tone: Tone, case: Case) -> Self {
        Self { base, tone, case }
    }

    /// The base vowel (root + shape) of this vowel.
    pub const fn base(self) -> BaseVowel {
        self.base
    }

    /// The root letter of this vowel.
    pub const fn root(self) -> RootVowel {
        self.base.root()
    }

    /// The diacritical shape of this vowel.
    pub const fn shape(self) -> Shape {
        self.base.shape()
    }

    pub const fn tone(self) -> Tone {
        self.tone
    }

    pub const fn case(self) -> Case {
        self.case
    }

    pub const fn has_shape(self) -> bool {
        !matches!(self.base.shape(), Shape::None)
    }

    pub const fn has_tone(self) -> bool {
        !matches!(self.tone, Tone::Flat)
    }

    /// Semantic vowel with `shape` replacing the current one, keeping the root.
    ///
    /// Returns `None` when the `(root, shape)` pair has no Vietnamese vowel
    /// (e.g. [`Shape::Stroke`], which never applies to a vowel).
    pub fn with_shape(self, shape: Shape) -> Option<Self> {
        Some(Self {
            base: BaseVowel::from_parts(self.base.root(), shape)?,
            tone: self.tone,
            case: self.case,
        })
    }

    /// Semantic vowel with `tone` replacing the current one.
    pub const fn with_tone(self, tone: Tone) -> Self {
        Self {
            base: self.base,
            tone,
            case: self.case,
        }
    }

    /// Semantic vowel with its tone cleared.
    pub const fn without_tone(self) -> Self {
        Self {
            base: self.base,
            tone: Tone::Flat,
            case: self.case,
        }
    }

    /// Semantic vowel with `case` replacing the current one.
    pub const fn with_case(self, case: Case) -> Self {
        Self {
            base: self.base,
            tone: self.tone,
            case,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_all_combinations() {
        const BASES: [BaseVowel; 12] = [
            BaseVowel::OHorn,
            BaseVowel::ECircumflex,
            BaseVowel::ABreve,
            BaseVowel::OCircumflex,
            BaseVowel::ACircumflex,
            BaseVowel::UHorn,
            BaseVowel::A,
            BaseVowel::O,
            BaseVowel::E,
            BaseVowel::I,
            BaseVowel::U,
            BaseVowel::Y,
        ];
        for &base in &BASES {
            for &tone in &[
                Tone::Flat,
                Tone::Acute,
                Tone::Grave,
                Tone::Hook,
                Tone::Tilde,
                Tone::Dot,
            ] {
                for &case in &[Case::Lower, Case::Upper] {
                    let vowel = Vowel::new(base, tone, case);
                    assert_eq!(vowel.base(), base);
                    assert_eq!(vowel.tone(), tone);
                    assert_eq!(vowel.case(), case);
                    assert_eq!(vowel.has_shape(), base.shape() != Shape::None);
                    assert_eq!(vowel.has_tone(), tone != Tone::Flat);
                }
            }
        }
    }

    #[test]
    fn stroke_is_not_a_vowel_shape() {
        let plain = Vowel::new(BaseVowel::A, Tone::Flat, Case::Lower);
        assert_eq!(plain.with_shape(Shape::Stroke), None);
    }

    #[test]
    fn transformations_update_semantics() {
        let plain = Vowel::new(BaseVowel::A, Tone::Flat, Case::Lower);
        let breve = plain.with_shape(Shape::Breve).unwrap();
        assert_eq!(breve.base(), BaseVowel::ABreve);
        assert_eq!(breve.root(), RootVowel::A);

        let acute = breve.with_tone(Tone::Acute);
        assert_eq!(acute.tone(), Tone::Acute);
        assert_eq!(acute.with_tone(Tone::Acute), acute);

        assert_eq!(acute.without_tone(), breve);
        assert_eq!(acute.without_tone().case(), Case::Lower);
    }

    #[test]
    fn case_is_preserved_through_transformations() {
        let upper = Vowel::new(BaseVowel::ABreve, Tone::Acute, Case::Upper);
        assert_eq!(upper.case(), Case::Upper);
        assert_eq!(upper.without_tone().case(), Case::Upper);
        let lowered = upper.with_case(Case::Lower);
        assert_eq!(lowered.case(), Case::Lower);
        assert_eq!(lowered.root(), RootVowel::A);
    }
}