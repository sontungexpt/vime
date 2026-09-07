//! Vietnamese syllable-composition rules.
//!
//! When a Vietnamese syllable is typed, its **onset** (phụ âm đầu) may
//! restrict which vowel letters it can fuse with, and its **vowel sequence**
//! (nguyên âm / vần) determines:
//!
//! - whether the nucleus is a valid Vietnamese contour,
//! - which vowel letter carries the tone mark (`tone_target_idx`), and
//! - which closing consonants (coda) the contour accepts.
//!
//! Every check here is O(1): sets are bitmasks and the vowel-contour logic is
//! a `match` over slice patterns.

use super::coda::Coda;
use super::config::ParsingMode;
use super::onset::Onset;
use super::vowel::BaseVowel;

/// A bitmask over [`BaseVowel`]s.
///
/// Bit `i` is set iff the [`BaseVowel`] whose priority ID equals `i` is in the
/// set. Because [`BaseVowel`] discriminants *are* the priority IDs
/// (`OHorn = 0 .. Y = 11`), membership is `1 << vowel.id()` — no lookup table
/// needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VowelSet(u16);

impl VowelSet {
    /// The empty set: no vowel is allowed.
    pub const NONE: Self = Self(0);

    /// Every base vowel at once. All 12 priorities `0..=11` fit in `u16`.
    pub const ALL: Self = Self((1 << BaseVowel::LEN) - 1);

    /// Front vowels — those articulated toward the front of the mouth
    /// (`i`, `e`, `ê`). Only these may follow the graphemes written with
    /// `k`, `gh` and `ngh`.
    pub const FRONT: Self = Self(
        (1 << BaseVowel::I.id()) | (1 << BaseVowel::E.id()) | (1 << BaseVowel::ECircumflex.id()),
    );

    /// The complement of [`Self::FRONT`]: every non-front vowel.
    pub const NON_FRONT: Self = Self(Self::ALL.0 & !Self::FRONT.0);

    /// Only `u`. `q` must always be followed by `u` in Vietnamese.
    pub const ONLY_U: Self = Self(1 << BaseVowel::U.id());

    /// Checks whether `vowel` is a member of this set.
    #[inline(always)]
    pub const fn contains(self, vowel: BaseVowel) -> bool {
        (self.0 & (1 << vowel.id())) != 0
    }
}

/// Checks whether `onset` may combine with the first vowel of the nucleus.
///
/// In `Strict` mode the spelling-to-sound constraints of Vietnamese apply;
/// `Loose` mode accepts anything (teen code, loan words, free typing).
///
/// The rules encode the three Vietnamese writing conventions:
///
/// - `k`, `gh`, `ngh` are used **only before front vowels** (`i`, `e`, `ê`);
/// - `c`, `g`, `ng` are used **only before non-front vowels**
///   (`a`, `ă`, `â`, `o`, `ô`, `ơ`, `u`, `ư`, `y`);
/// - `q` is **always followed by `u`**.
///
/// Everything else is unrestricted.
#[inline]
pub const fn match_onset_rule(onset: Onset, first_vowel: BaseVowel, mode: ParsingMode) -> bool {
    if matches!(mode, ParsingMode::Loose) {
        return true;
    }

    use Onset::*;
    use VowelSet as VS;

    let valid_set = match onset {
        // No consonant → any vowel is fine.
        None => VS::ALL,
        // Write "ki", "kê", "ghi", "nghi" — never before a, o, ô, ...
        K | Gh | Ngh => VS::FRONT,
        // Write "ca", "gô", "ngư" — the h-less graphemes forbid front vowels.
        C | G | Ng => VS::NON_FRONT,
        // "qua", "quy", "quê" — the `u` belongs to the QU onset.
        QU => VS::ALL,
        _ => VS::ALL,
    };

    valid_set.contains(first_vowel)
}

/// A bitmask over [`Coda`]s.
///
/// Bit `coda as u16` is set iff that coda may close the current vowel contour.
/// With only 9 codas (`0..=8`) a single `u16` holds every subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CodaSet(u16);

impl CodaSet {
    /// Open syllable — the only "coda" is having none.
    pub const NONE: Self = Self(1 << Coda::None as u16);

    /// Every possible closing consonant.
    pub const ALL: Self = Self((1 << 9) - 1);

    /// Central (non-palatal) codas: `c`, `m`, `n`, `p`, `t`, `ng` plus the
    /// open form. Used by contours that can never finish in `ch`/`nh`
    /// (e.g. the close diphthongs `iê`, `uô`, `ươ`).
    pub const CENTER: Self = Self(
        (1 << Coda::None as u16)
            | (1 << Coda::C as u16)
            | (1 << Coda::M as u16)
            | (1 << Coda::N as u16)
            | (1 << Coda::P as u16)
            | (1 << Coda::T as u16)
            | (1 << Coda::Ng as u16),
    );

    /// Palatal-ish codas `ch`, `nh` plus the open form, for contours that may
    /// end in them (e.g. `oe`, `uy` → "hoe", "huỳnh").
    pub const FRONT: Self =
        Self((1 << Coda::None as u16) | (1 << Coda::Ch as u16) | (1 << Coda::Nh as u16));

    /// Checks whether `coda` may close a contour described by this set.
    #[inline(always)]
    pub const fn contains(self, coda: Coda) -> bool {
        (self.0 & (1 << coda as u16)) != 0
    }
}

/// Whether a vowel contour is a complete valid spelling or merely a prefix
/// that still needs more letters to become one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SequenceState {
    /// The contour is ready to accept a tone mark and a coda.
    Valid,
    /// The contour is an *intermediate* spelling (e.g. "ie" on the way to
    /// "iê") that must receive more vowels before it can finish.
    Transitional,
}

/// The outcome of matching a vowel sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VowelSequenceRule {
    /// Whether the contour is complete (`Valid`) or still building
    /// (`Transitional`), or `None` from the match if it is illegal.
    pub state: SequenceState,
    /// Index into the vowel slice of the letter that must carry the tone mark.
    /// `0` marks the first (and for single vowels, only) vowel.
    pub tone_target_idx: usize,
    /// The set of codas this contour may take, as a bitmask.
    pub valid_codas: CodaSet,
}

/// O(1) pattern match on a vowel slice that decides the fate of the contour.
///
/// Returns `None` if the sequence is not a legal Vietnamese vowel contour.
/// Otherwise returns the [`SequenceState`], the letter that receives the tone
/// mark, and the [`CodaSet`] the contour may close with.
#[inline]
pub const fn match_vowel_sequence(vowels: &[BaseVowel]) -> Option<VowelSequenceRule> {
    use BaseVowel::*;
    use CodaSet as CS;
    use SequenceState::*;

    let (state, tone_target, codas) = match vowels {
        // ─────────────── 1. Single vowels ───────────────
        // Any one of the 12 base vowels is a complete nucleus that may take
        // any coda. The tone always lands on this (the only) vowel.
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
        | [Y] => (Valid, 0, CS::ALL),

        // ─────────────── 2. Open diphthongs ───────────────
        // Two-vowel contours that can never close: the word must end right
        // after the second vowel (e.g. "hai", "câu", "mèo", "bay").
        // Tone mark still falls on the first letter; no coda is allowed.
        [A, I]
        | [A, O]
        | [A, U]
        | [A, Y]
        | [ACircumflex, U]
        | [ACircumflex, Y]
        | [E, O]
        | [E, U]
        | [OHorn, I]
        | [OHorn, U]
        | [UHorn, I] => (Valid, 0, CS::NONE),

        // ─────────────── 3. The iê/ia · uô/ua · ươ/ưa family ───────────────
        // The two vowels "iê", "uô", "ươ" are written as open "ia", "ua",
        // "ưa" when the syllable ends without a coda. With a coda they must
        // keep the closed spelling; the tone mark rides on the *second* letter
        // and the closing is central-only.
        //
        // Contour            coda?     example
        // ---------          -----     -------------
        // "ia" / "ie"        no        "chia", "mua", "mưa" (valid, open)
        // "iê" / "uô" / "ươ" yes       "tiền", "cuốn", "cường" (center codas)

        // "ia" before nothing: complete, open (e.g. "chia", "vía").
        [I, A] => (Valid, 0, CS::NONE),
        // "iê" before a coda: complete, tone on ê, central closing only.
        [I, ECircumflex] => (Valid, 1, CS::CENTER),
        // "ie": legal only as an intermediate step toward "iê".
        [I, E] => (Transitional, 0, CS::CENTER),

        // Same three-way split for uô/ua ("mua", "cuốn")...
        [U, A] => (Valid, 0, CS::NONE),
        [U, OCircumflex] => (Valid, 1, CS::CENTER),
        [U, O] => (Transitional, 0, CS::CENTER),

        // ...and for ươ/ưa ("thưa", "cường").
        [UHorn, A] => (Valid, 0, CS::NONE),
        [UHorn, OHorn] => (Valid, 1, CS::CENTER),
        [UHorn, O] => (Transitional, 0, CS::CENTER),

        // ─────────────── 4. Other closing diphthongs ───────────────
        // "oa" → open or central closing, tone on 'a' ("hoa", "xoạt").
        [O, A] => (Valid, 1, CS::CENTER),
        // "oe" → closes in ch/nh (or open), tone on 'e' ("hoè", "toềnh"? no: "xoen").
        [O, E] => (Valid, 1, CS::FRONT),
        // "uy" → closes in ch/nh (or open), tone on 'y' ("huy", "huỳnh").
        [U, Y] => (Valid, 1, CS::FRONT),

        // ─────────────── 5. Triphthongs ───────────────
        // Three-vowel contours. These always carry the tone on a specific
        // letter; their closing sets are fixed per contour.
        //
        // "iêu" – no closing ("yêu", "biêu").
        [I, ECircumflex, U] => (Valid, 1, CS::NONE),
        // "uyên" – tone on ê, central closing ("khuyên", "tuyên").
        [U, Y, ECircumflex] => (Valid, 2, CS::CENTER),
        // "ươi" / "ươu" – no closing ("nưới"? no: "cưới", "ươu").
        [UHorn, OHorn, I] | [UHorn, OHorn, U] => (Valid, 1, CS::NONE),
        // "oai" / "oao" – no closing ("xoaí"? no: "khoai", "ngoao").
        [O, A, I] | [O, A, O] => (Valid, 1, CS::NONE),

        // Anything else is not a Vietnamese contour.
        _ => return None,
    };

    Some(VowelSequenceRule {
        state,
        tone_target_idx: tone_target,
        valid_codas: codas,
    })
}
