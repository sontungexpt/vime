use super::vowel::BaseVowel;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceStatus {
    Dead,
    Valid,
    InComplete,
}

#[inline]
pub const fn match_vowel_sequence(vowels: &[BaseVowel]) -> SequenceStatus {
    use BaseVowel::*;
    use SequenceStatus::*;

    match vowels {
        // ─────────────────── a ───────────────────
        [A, I] | [A, O] | [A, U] | [A, Y] | [ACircumflex, U] | [ACircumflex, Y] => Valid,

        // ─────────────────── i ───────────────────
        [I, A] => Valid,
        [I, E] => InComplete,
        [I, ECircumflex] => Valid,

        [I, ECircumflex, U] => Valid,

        // ─────────────────── y ───────────────────
        [Y, E] => InComplete,
        [Y, ECircumflex] => Valid,

        [Y, E, U] => InComplete,
        [Y, ECircumflex, U] => Valid,

        // ─────────────────── e ───────────────────
        [E, O] | [E, U] => Valid,

        // ─────────────────── o ───────────────────
        [O, A] | [O, E] => Valid,

        [O, A, I] | [O, A, O] => Valid,

        // ─────────────────── u ───────────────────
        [U, A] => Valid,
        [U, O] => InComplete,
        [U, OCircumflex] => Valid,

        [U, Y] => Valid,
        [U, Y, E] => InComplete,
        [U, Y, ECircumflex] => Valid,

        [U, O, I] => InComplete,
        [U, OCircumflex, I] => Valid,

        // ─────────────────── ư ───────────────────
        [UHorn, A] => Valid,
        [UHorn, O] => InComplete,
        [UHorn, OHorn] => Valid,

        [UHorn, I] | [UHorn, U] => Valid,

        [UHorn, OHorn, I] | [UHorn, OHorn, U] => Valid,

        // ─────────────────── Single vowels ───────────────────
        [A]
        | [ABreve]
        | [ACircumflex]
        | [E]
        | [ECircumflex]
        | [I]
        | [O]
        | [OCircumflex]
        | [OHorn]
        | [U]
        | [UHorn]
        | [Y] => Valid,

        // ─────────────────── Invalid ───────────────────
        _ => Dead,
    }
}
