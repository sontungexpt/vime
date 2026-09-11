use crate::{
    keymapping::{KeyMapping, KeyTarget},
    phonology::{
        decode_vowel,
        rule::{self, check_nucleus_validity, NucleusStatus},
        BaseVowel, Case, Coda, Onset, Shape, Tone,
    },
    BufferChar, RootVowel,
};

use super::syllable::{Cased, Syllable};

/// Whether `c` is an ASCII consonant character.
///
/// Consonants are the ASCII letters minus the vowels `a, e, i, o, u` (and minus
/// `y`, which the Vietnamese vowel codec owns). `đ` / `Đ` are also onsets.
#[inline(always)]
const fn is_ascii_consonant(c: char) -> bool {
    matches!(
        c,
        'b'..='d' | 'f'..='h' | 'j'..='n' | 'p'..='t' | 'v'..='x' | 'z' |
        'B'..='D' | 'F'..='H' | 'J'..='N' | 'P'..='T' | 'V'..='X' | 'Z' |
        'đ' | 'Đ'
    )
}

/// Outcome of applying a transform key to the current syllable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformEffect {
    /// The transform was applied to the syllable.
    Applied,
    /// The transform undid an existing mark; the key falls through as a literal.
    Reverted,
    /// The key could not act as a transform here.
    NotApplicable,
}

/// The parsing phase the syllable is currently in.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsePhase {
    /// Reading the initial consonant(s).
    #[default]
    Onset,
    /// Reading the vowel nucleus.
    Vowel,
    /// Reading the final consonant(s).
    Coda,
}

/// Why a syllable was rejected.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadReason {
    /// Unknown / unreachable failure.
    #[default]
    Unknown,
    /// The consonant cluster is not a valid Vietnamese onset.
    InvalidOnset,
    /// The vowel nucleus violates the Vietnamese vowel-rule table.
    InvalidVowelSequence,
    /// The final consonant cluster is not a valid Vietnamese coda.
    InvalidCoda,
    /// A character that is neither a consonant nor a vowel (`?`, `,`, `/`, …).
    InvalidCharacter,
}

/// Overall result of a parse.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    /// The syllable is still being built and may become valid.
    #[default]
    Incomplete,
    /// The syllable is a complete, valid Vietnamese syllable.
    Valid,
    /// The syllable can never become valid.
    Dead(DeadReason),
}

/// A point-in-time copy of the parser state, for undo / commit workflows.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ParseSnapshot {
    phase: ParsePhase,
    syllable: Syllable,
    status: ParseStatus,
}

/// Incremental syllable parser driven by a [`KeyMapping`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser<'a, KM: KeyMapping> {
    syllable: Syllable,
    phase: ParsePhase,
    status: ParseStatus,
    mapping: &'a KM,
}

impl<'a, KM: KeyMapping> Parser<'a, KM> {
    /// Creates a parser backed by `mapping`, starting in the `Onset` phase.
    #[inline(always)]
    pub fn new(mapping: &'a KM) -> Self {
        Self {
            syllable: Syllable::default(),
            phase: ParsePhase::Onset,
            status: ParseStatus::Incomplete,
            mapping,
        }
    }

    /// Returns a copy of the current parser state.
    #[inline(always)]
    pub fn snapshot(&self) -> ParseSnapshot {
        ParseSnapshot {
            phase: self.phase,
            syllable: self.syllable.clone(),
            status: self.status,
        }
    }

    /// Clears the syllable and returns the parser to the `Onset` phase.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.syllable = Syllable::default();
        self.phase = ParsePhase::Onset;
        self.status = ParseStatus::Incomplete;
    }

    /// Returns the syllable built so far.
    #[inline(always)]
    pub const fn syllable(&self) -> &Syllable {
        &self.syllable
    }

    /// Returns the current parse status.
    #[inline(always)]
    pub const fn status(&self) -> ParseStatus {
        self.status
    }

    /// Returns the phase the parser is currently in.
    #[inline(always)]
    pub const fn phase(&self) -> ParsePhase {
        self.phase
    }

    /// Marks the syllable as dead for `reason`.
    #[inline(always)]
    const fn kill(&mut self, reason: DeadReason) -> ParseStatus {
        self.status = ParseStatus::Dead(reason);
        self.status
    }

    /// Advances the parser to the given phase.
    #[inline(always)]
    const fn set_phase(&mut self, phase: ParsePhase) {
        self.phase = phase;
    }

    /// Pushes one input buffer character and returns the new status.
    ///
    /// No character is consumed once the syllable is dead.
    #[inline]
    pub fn push(&mut self, input: BufferChar) -> ParseStatus {
        if let ParseStatus::Dead(_) = self.status {
            return self.status;
        }

        match self.phase {
            ParsePhase::Onset => self.push_onset(input),
            ParsePhase::Vowel => self.push_vowel(input),
            ParsePhase::Coda => self.push_coda(input),
        }
    }

    /// Pushes a sequence of input characters.
    ///
    /// Returns the resulting status and, when the parse is dead, the index of
    /// the character at which it first died. A parser that was already dead
    /// before the call yields `None`.
    pub fn parse(&mut self, chars: &[BufferChar]) -> (ParseStatus, Option<usize>) {
        if let ParseStatus::Dead(_) = self.status {
            return (self.status, None);
        }

        for (index, ch) in chars.iter().enumerate() {
            if let status @ ParseStatus::Dead(_) = self.push(*ch) {
                return (status, Some(index));
            }
        }

        (self.status, None)
    }

    /// Routes an input character to the onset literal / transform handlers.
    #[inline]
    fn push_onset(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_onset_literal(ch),
            BufferChar::Transform(key) => self.push_onset_transform(key),
        }
    }

    /// Handles a literal character while in the `Onset` phase.
    ///
    /// Consonants extend the onset. The first vowel opens the nucleus,
    /// treating a `u` after a lone `q` as the `qu` onset.
    fn push_onset_literal(&mut self, ch: char) -> ParseStatus {
        let syllable = &mut self.syllable;

        // Any ASCII consonant is appended to the onset.
        if is_ascii_consonant(ch) {
            syllable.onset_chars.push(ch);
            return self.status;
        }
        // Otherwise, if it is a vowel
        else if let Some((base, tone, case)) = decode_vowel(ch) {
            // qu
            // A `u` following a lone `q` is kept as the onset of `qu`.
            // Only the plain `u` counts here: a precomposed `ư` is a vowel.
            if base == BaseVowel::U
                && syllable.onset_chars.len() == 1
                && syllable.onset_chars[0].to_ascii_lowercase() == 'q'
            {
                // Treat `u` as an onset consonant in this special case.
                syllable.onset_chars.push(ch);
                return self.status;
            }

            // Handle the first vowel encountered.
            //
            // A zero onset is also valid: "a", "ă", "â", ...
            if syllable.onset_chars.len() > 0 {
                match Onset::from_chars(&syllable.onset_chars) {
                    Ok(kind) => {
                        syllable.onset = Some(kind);
                    }
                    Err(_) => return self.kill(DeadReason::InvalidOnset),
                }
            }

            syllable.vowels.push(Cased { value: base, case });
            syllable.tone = tone;

            self.set_phase(ParsePhase::Vowel);
            return self.status;
        }

        // Neither a consonant nor a vowel: stray keys such as `?` or `/`.
        self.kill(DeadReason::InvalidCharacter)
    }

    /// Handles a transform key while in the `Onset` phase.
    ///
    /// Only the D-stroke is meaningful here; every other key is treated as a
    /// literal. A revert undoes the stroke and then falls back to a literal.
    fn push_onset_transform(&mut self, key: char) -> ParseStatus {
        // Any key that is not a stroke key is treated as an ordinary letter.
        if !self.mapping.stroke(key) {
            return self.push_onset_literal(key);
        }

        match self.try_d_stroke() {
            TransformEffect::Applied => {
                return self.status;
            }
            TransformEffect::Reverted => {
                // The stroke was undone, so the key falls back to a literal.
                // e.g. toggling 'đ' back to 'd' and adding the key gives "dd".
                return self.push_onset_literal(key);
            }
            TransformEffect::NotApplicable => {
                return self.push_onset_literal(key);
            }
        }
    }

    /// With "gi" followed by another vowel, the 'i' leaves the vowel
    /// sequence and joins the onset so "gi" becomes the onset `Onset::Gi`.
    #[inline(always)]
    fn resolve_gi_onset(&mut self) -> bool {
        let syllable = &mut self.syllable;

        let Some(i) = syllable.vowels.pop() else {
            return false;
        };

        syllable
            .onset_chars
            .push(if i.case == Case::Lower { 'i' } else { 'I' });
        syllable.onset = Some(Onset::Gi);

        true
    }

    /// Routes an input character to the vowel literal / transform handlers.
    #[inline]
    fn push_vowel(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_vowel_literal(ch),
            BufferChar::Transform(key) => self.push_vowel_transform(key),
        }
    }

    /// Appends a precomposed vowel to the nucleus if it does not clash with
    /// the tone already held by the syllable.
    #[inline(always)]
    fn push_vowel_precomposed(&mut self, base: BaseVowel, tone: Tone, case: Case) -> bool {
        let syllable = &mut self.syllable;

        // A precomposed vowel carrying a tone must not conflict with the tone
        // already on the syllable. e.g. "á" is already typed and a raw "ắ" is
        // pushed straight into the buffer - the two tones would collide.
        if tone != Tone::Flat && syllable.tone != Tone::Flat {
            return false; // push failed
        }

        syllable.vowels.push(Cased { value: base, case });

        if tone != Tone::Flat {
            syllable.tone = tone;
        }

        true // push succeeded
    }

    /// Handles a literal character while in the `Vowel` phase.
    ///
    /// A vowel is added to the nucleus; a consonant closes the nucleus and
    /// moves on to the `Coda` phase.
    fn push_vowel_literal(&mut self, ch: char) -> ParseStatus {
        if let Some((base, tone, case)) = decode_vowel(ch) {
            // ---------------------------------------------------------
            // Special case: gi
            // A syllable prefix of "gi" is ambiguous: if another vowel
            // follows the 'i', "gi" becomes the onset; otherwise 'g'
            // stays the onset and the 'i' is the nucleus.
            // ---------------------------------------------------------
            if self.syllable.onset == Some(Onset::G)
                // Only 'i' is in the vowel sequence so far
                && self.syllable.vowels[0].value == BaseVowel::I
                && self.syllable.vowels.len() == 1
            {
                // "gi" is already typed and another vowel follows,
                // so "gi" now becomes the onset.
                if !self.resolve_gi_onset() {
                    // Unreachable in practice
                    return self.kill(DeadReason::Unknown);
                }
            }

            // Vietnamese vowel sequence supports at most 3 vowels.
            if self.syllable.vowels.len() >= 3 {
                return self.kill(DeadReason::InvalidVowelSequence);
            }
            // Append the new vowel to the sequence
            else if !self.push_vowel_precomposed(base, tone, case) {
                return self.kill(DeadReason::InvalidVowelSequence);
            }

            // `normalize_uo` validates the length itself and only acts once
            // there are at least two vowels - which is exactly when a third
            // vowel is being pushed here.
            self.normalize_uo();

            return self.status;
        }

        // ---------------------------------------------------------
        // Not a vowel.
        //
        // A consonant starts the coda - switch phase.
        // ---------------------------------------------------------
        if !is_ascii_consonant(ch) {
            // Not an ASCII consonant: treat it as a special character.
            return self.kill(DeadReason::InvalidCharacter);
        }

        // An already-dead vowel sequence cannot be rescued once the coda has
        // started, since the sequence has fused with the coda.
        // e.g. 'chơa' + 'c' -> 'chơac' (invalid coda).
        if NucleusStatus::Dead == self.validate_nucleus::<3>() {
            return self.kill(DeadReason::InvalidVowelSequence);
        }

        // `normalize_uo` validates the length itself and only acts once there
        // are at least two vowels - which is exactly when a coda character is
        // being pushed here.
        self.normalize_uo();

        self.syllable.coda_chars.push(ch);
        self.set_phase(ParsePhase::Coda);
        self.status
    }

    /// Resolves a tone or D-stroke transform key.
    ///
    /// Returns the transform effect, or `None` if the key is neither a tone
    /// nor a stroke key.
    #[inline(always)]
    fn try_common_transform(&mut self, key: char) -> Option<TransformEffect> {
        // ---------------------------------------------------------
        // 1. Tone
        // ---------------------------------------------------------
        if let Some(tone) = self.mapping.tone(key) {
            return Some(self.try_tone(tone));
        }

        // ---------------------------------------------------------
        // 2. D-stroke
        // ---------------------------------------------------------
        if self.mapping.stroke(key) {
            return Some(self.try_d_stroke());
        }

        None
    }

    /// Handles a transform key while in the `Vowel` phase.
    ///
    /// Tries tones, then the D-stroke, then vowel shapes.
    fn push_vowel_transform(&mut self, key: char) -> ParseStatus {
        // `Some` means the key resolved into a tone or a stroke.
        if let Some(effect) = self.try_common_transform(key) {
            match effect {
                TransformEffect::Applied => {
                    return self.status;
                }
                TransformEffect::Reverted => {
                    return self.push_vowel_literal(key);
                }
                TransformEffect::NotApplicable => {
                    return self.push_vowel_literal(key);
                }
            }
        }

        // No tone or stroke effect: try a shape instead.

        // ---------------------------------------------------------
        // 2. Shape
        //
        // Find the nearest vowel that can take the shape.
        // ---------------------------------------------------------

        match self.try_shape_transform(key) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_vowel_literal(key)
            }
        }
    }

    /// Routes an input character to the coda literal / transform handlers.
    #[inline]
    fn push_coda(&mut self, input: BufferChar) -> ParseStatus {
        match input {
            BufferChar::Literal(ch) => self.push_coda_literal(ch),
            BufferChar::Transform(key) => self.push_coda_transform(key),
        }
    }

    /// Handles a literal character while in the `Coda` phase.
    fn push_coda_literal(&mut self, ch: char) -> ParseStatus {
        if self.syllable.coda_chars.len() > Coda::MAX_CODA_LEN {
            return self.kill(DeadReason::InvalidCoda);
        }

        if !is_ascii_consonant(ch) {
            // A vowel that arrives inside the coda is junk.
            // e.g. in 'tiếnog', the 'o' between 'n' and 'g' is junk.
            return self.kill(DeadReason::InvalidCharacter);
        }

        self.syllable.coda_chars.push(ch);

        // Re-validate right away to catch a dead coda.
        match Coda::from_chars(&self.syllable.coda_chars) {
            Ok(kind) => {
                self.syllable.coda = Some(kind);
                self.status
            }

            Err(_) => self.kill(DeadReason::InvalidCoda),
        }
    }

    /// Handles a transform key while in the `Coda` phase.
    ///
    /// Tone, D-stroke and shape transforms are forwarded to the vowel
    /// handlers; anything else is treated as a literal.
    fn push_coda_transform(&mut self, key: char) -> ParseStatus {
        if let Some(effect) = self.try_common_transform(key) {
            return match effect {
                TransformEffect::Applied => self.status,
                TransformEffect::Reverted | TransformEffect::NotApplicable => {
                    self.push_coda_literal(key)
                }
            };
        }

        match self.try_shape_transform(key) {
            TransformEffect::Applied => self.status,
            TransformEffect::Reverted | TransformEffect::NotApplicable => {
                self.push_coda_literal(key)
            }
        }
    }

    /// Whether the nucleus starts with an unmarked `u o` pair.
    #[inline(always)]
    fn is_uo_first(&self) -> bool {
        return self.syllable.vowels.len() > 1
            && self.syllable.vowels[0].value.root() == RootVowel::U
            && self.syllable.vowels[1].value.root() == RootVowel::O;
    }

    /// Applies a Horn shape to the `u o` prefix (requires at least 2 vowels).
    #[inline]
    fn try_uo_horn(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;

        return match (vowels[0].value, vowels[1].value) {
            // ươ -> uow
            //
            // This always reverts, even with a third vowel.
            // e.g.:
            //
            //  'ươ' -> 'uow'
            //  'ươu' -> 'uouw'
            //  'ươơ' -> 'uoơw'
            //  'ươư' -> 'uoưw'
            //  'ươi' -> 'uoiw'
            //  'ươa' -> 'uoaw'
            //  'ươă' -> 'uoăw'
            //  'ươy' -> 'uoyw'
            //  'ươe' -> 'uoew'
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                // Considering if we should do this or not because this case will be revert to ascii by renderer
                // if vowels.len() == 3
                //     && (vowels[2].value != BaseVowel::U && vowels[2].value != BaseVowel::I)
                // {
                //     return TransformEffect::NotApplicable;
                // }

                self.syllable.vowels[0].value = BaseVowel::U;
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Reverted;
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => {
                return TransformEffect::NotApplicable;
            }

            // ưo -> ươ
            (BaseVowel::UHorn, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }

            // uo -> uơ
            (BaseVowel::U, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }
            // uơ -> ươ
            (BaseVowel::U, BaseVowel::OHorn) => {
                return self.try_vowel_shape(0, Shape::Horn);
            }
            // uô -> uơ
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                return self.try_vowel_shape(1, Shape::Horn);
            }
            _ => TransformEffect::NotApplicable,
        };
    }

    /// Applies a Circumflex shape to the `u o` prefix (requires at least 2 vowels).
    #[inline]
    fn try_uo_circumflex(&mut self) -> TransformEffect {
        let vowels = &self.syllable.vowels;
        let first = vowels[0].value;
        let second = vowels[1].value;

        return match (first, second) {
            // ươ -> uô
            (BaseVowel::UHorn, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::U;

                // Try putting a circumflex on ơ.
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    // Cannot happen: ư has the horn shape, so a circumflex never toggles it
                    TransformEffect::Reverted => TransformEffect::Reverted,
                    TransformEffect::NotApplicable => {
                        // rollback
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // ưô -> not applicable
            (BaseVowel::UHorn, BaseVowel::OCircumflex) => {
                return TransformEffect::NotApplicable;
            }

            // ưo -> uô
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.syllable.vowels[0].value = BaseVowel::U;
                // Try adding a circumflex to o.
                match self.try_vowel_shape(1, Shape::Circumflex) {
                    TransformEffect::Applied => TransformEffect::Applied,
                    // Cannot happen: a circumflex never toggles the ư on the left
                    TransformEffect::Reverted => TransformEffect::Reverted,
                    TransformEffect::NotApplicable => {
                        // rollback
                        self.syllable.vowels[0].value = first;
                        TransformEffect::NotApplicable
                    }
                }
            }

            // uo -> uô
            (BaseVowel::U, BaseVowel::O) => {
                return self.try_vowel_shape(1, Shape::Circumflex);
            }

            // uơ -> uô
            (BaseVowel::U, BaseVowel::OHorn) => {
                return self.try_vowel_shape(1, Shape::Circumflex);
            }

            // uô -> undo (revert to "uo")
            (BaseVowel::U, BaseVowel::OCircumflex) => {
                // Considering if we should do this or not because this case will be revert to ascii by renderer
                // if vowels.len() == 3 && vowels[2].value != BaseVowel::I
                // {
                //     return TransformEffect::NotApplicable;
                // }
                self.syllable.vowels[1].value = BaseVowel::O;
                return TransformEffect::Reverted;
            }
            _ => TransformEffect::NotApplicable,
        };
    }

    /// Applies or toggles a tone on the syllable.
    ///
    /// Tapping the same tone again toggles the syllable back to `Flat`;
    /// a different tone replaces the current one.
    #[inline(always)]
    fn try_tone(&mut self, tone: Tone) -> TransformEffect {
        let syllable = &mut self.syllable;

        if syllable.vowels.is_empty() {
            return TransformEffect::NotApplicable;
        }

        // Same tone -> toggle back to Flat.
        if syllable.tone == tone {
            syllable.tone = Tone::Flat;
            return TransformEffect::Reverted;
        }

        // Different tone -> replace the current tone.
        syllable.tone = tone;
        TransformEffect::Applied
    }

    /// Toggles the D-stroke on the last onset character.
    #[inline]
    fn try_d_stroke(&mut self) -> TransformEffect {
        for ch in self.syllable.onset_chars.iter_mut().rev() {
            match *ch {
                'd' => {
                    *ch = 'đ';
                    return TransformEffect::Applied;
                }
                'D' => {
                    *ch = 'Đ';
                    return TransformEffect::Applied;
                }
                'đ' => {
                    *ch = 'd';
                    return TransformEffect::Reverted;
                }
                'Đ' => {
                    *ch = 'D';
                    return TransformEffect::Reverted;
                }
                _ => {}
            }
        }

        TransformEffect::NotApplicable
    }

    /// Tries to apply `key` as a shape transform on the vowel sequence,
    /// scanning from the last vowel backwards.
    fn try_shape_transform(&mut self, key: char) -> TransformEffect {
        let vseq_len = self.syllable.vowels.len();

        for index in (0..vseq_len).rev() {
            let base = self.syllable.vowels[index].value;

            let Some(shape) = self.mapping.shape(key, KeyTarget::BaseVowel(base)) else {
                continue;
            };

            // Handle the special u / o cases.

            if shape == Shape::Horn && self.is_uo_first() {
                return self.try_uo_horn();
            } else if shape == Shape::Circumflex && self.is_uo_first() {
                return self.try_uo_circumflex();
            }

            let effect = self.try_vowel_shape(index, shape);
            // If that failed, keep trying the earlier vowels.
            if effect != TransformEffect::NotApplicable {
                return effect;
            }
        }

        TransformEffect::NotApplicable
    }

    /// Validates the vowel nucleus against the rule table, keeping only the
    /// first `N` vowels.
    #[inline(always)]
    fn validate_nucleus<const N: usize>(&self) -> NucleusStatus {
        let mut buf = [BaseVowel::A; N]; // Fixed-size N array on the stack (avoids dynamic allocation)
        let len = self.syllable.vowels.len().min(N);

        for (i, v) in self.syllable.vowels.iter().take(len).enumerate() {
            buf[i] = v.value;
        }

        // Slice exactly the used length and validate.
        check_nucleus_validity(&buf[..len])
    }

    /// Applies `shape` to the vowel at `index`, re-validating the nucleus.
    ///
    /// Applying the shape it already has reverts it; an invalid result rolls
    /// the vowel back.
    #[inline]
    fn try_vowel_shape(&mut self, index: usize, shape: Shape) -> TransformEffect {
        let old = self.syllable.vowels[index].value;

        // ---------------------------------------------------------
        // 1. Undo first.
        //
        // When the vowel already has exactly this shape, the state is
        // guaranteed valid, so the shape can simply be removed without
        // re-validating the candidate.
        // ---------------------------------------------------------
        if old.shape() == shape && shape != Shape::None {
            self.syllable.vowels[index].value = old.remove_shape();
            return TransformEffect::Reverted;
        }

        let Ok(new) = old.replace_shape(shape) else {
            // The shape does not form a valid sequence, e.g. the user
            // deliberately applied a forbidden combo such as a + horn.
            return TransformEffect::NotApplicable;
        };

        // Try applying the shape.
        self.syllable.vowels[index].value = new;

        // A single vowel needs no sequence validation.
        if self.syllable.vowels.len() < 2 {
            return TransformEffect::Applied;
        }

        match self.validate_nucleus::<3>() {
            rule::NucleusStatus::Valid => TransformEffect::Applied,
            rule::NucleusStatus::InComplete => TransformEffect::Applied,
            rule::NucleusStatus::Dead => {
                self.syllable.vowels[index].value = old;
                TransformEffect::NotApplicable
            }
        }
    }

    /// Normalizes an unmarked `u o` prefix that arrived without a shape key.
    ///
    /// `uơ → ươ` and `ưo → ươ` are folded once at least two vowels are present.
    #[inline]
    fn normalize_uo(&mut self) {
        let vowels = &self.syllable.vowels;
        if vowels.len() < 2 {
            return;
        }

        match (vowels[0].value, vowels[1].value) {
            // uơ → ươ
            (BaseVowel::U, BaseVowel::OHorn) => {
                self.syllable.vowels[0].value = BaseVowel::UHorn;
            }
            // ưo → ươ
            (BaseVowel::UHorn, BaseVowel::O) => {
                self.syllable.vowels[1].value = BaseVowel::OHorn;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
