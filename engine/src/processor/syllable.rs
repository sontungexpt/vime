//! Structural analysis of Vietnamese syllables over canonical ASCII raw text.
//!
//! Mental model:
//!
//! ```text
//! canonical ASCII raw
//!     ↓ parse once (single byte scan)
//! WordStructure { Onset, Vowel, Coda IDs, tone, byte spans }
//!     ↓ table lookups / bitmasks
//! validation · tone position · rendering
//! ```

use std::ops::Range;

use super::normalize::Decoder;
use super::rules::{rule, vowel_for, Coda, Onset, Vowel, BASE_LETTERS};
use super::tone::{main_index, read_tones, rescue_trailing_tone, tone_marker, Orthography};
use super::vowel::{atom_width, is_breve, is_circumflex, is_horn, read_run};
use crate::character::Tone;

/// The linguistic status of a partially typed syllable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceState {
    Invalid,
    Transitional,
    Valid,
}

/// Byte spans of the structural regions inside a canonical raw syllable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spans {
    pub onset: Range<usize>,
    pub vowels: Range<usize>,
    pub coda: Range<usize>,
}

/// The parsed structure of one canonical raw syllable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WordStructure {
    pub state: SequenceState,
    pub onset: Onset,
    pub(crate) vowel: Vowel,
    pub coda: Coda,
    /// Tone attached to the main vowel; `None` is untoned.
    pub tone: Option<Tone>,
    /// Main-vowel atom index for tone placement; valid words only.
    pub(crate) main_vowel: Option<u8>,
    pub spans: Spans,
}

impl WordStructure {
    fn invalid(length: usize) -> Self {
        Self {
            state: SequenceState::Invalid,
            onset: Onset::None,
            vowel: Vowel::A,
            coda: Coda::None,
            tone: None,
            main_vowel: None,
            spans: Spans {
                onset: 0..0,
                vowels: 0..0,
                coda: length..length,
            },
        }
    }
}

const ONSET_WINDOW: usize = 4;
const INITIATORS: &[u8] = b"aeiouyw";

#[inline]
fn is_initiator(byte: u8) -> bool {
    INITIATORS.contains(&byte)
}

/// Parses one canonical raw syllable.
///
/// Segmentation candidates are every vowel-start offset within reach of an
/// onset (`0..4`). The first fully valid candidate wins; otherwise the last
/// transitional candidate is reported; otherwise the word is invalid.
///
/// As a final pass, a word that is invalid because tone markers are stranded
/// inside its vowel run (`nguowf` + `i`) is repaired with
/// [`normalize::lift_markers`](super::normalize::lift_markers) and parsed once
/// more. Spans of a repaired word index the repaired letters, which are the
/// same length as the input.
pub fn parse(raw: &str, orthography: Orthography) -> WordStructure {
    let word = parse_once(raw.as_bytes(), orthography);
    if word.state != SequenceState::Invalid {
        return word;
    }
    let Some(repaired) = super::normalize::lift_markers(raw) else {
        return word;
    };
    parse_once(repaired.as_bytes(), orthography)
}

fn parse_once(bytes: &[u8], orthography: Orthography) -> WordStructure {
    let mut latest_transitional: Option<WordStructure> = None;

    for vs in 0..bytes.len().min(ONSET_WINDOW) {
        if !is_initiator(bytes[vs]) {
            continue;
        }
        let Some(onset) = Onset::identify(&bytes[..vs]) else {
            continue;
        };
        let Some(run) = read_run(bytes, vs) else {
            continue;
        };
        let Some(vowel) = vowel_for(run.atoms, run.count) else {
            continue;
        };
        let grammar = rule(vowel);

        let mut slot = run.end;
        let mut tone = read_tones(bytes, &mut slot);
        let mut coda_end = bytes.len();
        if Coda::identify(&bytes[slot..]).is_none() {
            if let Some((end, rescued)) = rescue_trailing_tone(bytes, slot) {
                tone = rescued;
                coda_end = end;
            }
        }
        let coda_range = slot..coda_end;
        let coda = Coda::identify(&bytes[coda_range.clone()]);

        let state = if coda_range.is_empty() {
            grammar.state
        } else {
            match coda.filter(|c| grammar.allows_coda(*c)) {
                Some(_) => SequenceState::Valid,
                None => SequenceState::Invalid,
            }
        };

        let has_coda = !coda_range.is_empty();
        let word = WordStructure {
            state,
            onset,
            vowel,
            coda: coda.unwrap_or(Coda::None),
            tone,
            main_vowel: (state == SequenceState::Valid)
                .then(|| main_index(grammar, orthography, has_coda)),
            spans: Spans {
                onset: 0..vs,
                vowels: vs..run.end,
                coda: coda_range,
            },
        };
        match state {
            SequenceState::Valid => return word,
            SequenceState::Transitional => latest_transitional = Some(word),
            SequenceState::Invalid => {}
        }
    }
    latest_transitional.unwrap_or_else(|| WordStructure::invalid(bytes.len()))
}

/// Re-serializes a parsed word into canonical form.
///
/// This merges duplicate horn markers, relocates trailing tone markers before
/// the coda, and deduplicates tone keys.
pub(crate) fn serialize(word: &WordStructure, letters: &str, out: &mut String) {
    out.push_str(&letters[word.spans.onset.clone()]);
    let grammar = rule(word.vowel);
    for index in 0..grammar.count as usize {
        let quality = grammar.atoms[index];
        let base = BASE_LETTERS[quality as usize];
        out.push(base);
        if is_circumflex(quality) {
            out.push(base);
        }
        if is_breve(quality) {
            out.push('w');
        }
        if is_horn(quality) && !grammar.atoms.get(index + 1).copied().is_some_and(is_horn) {
            out.push('w');
        }
    }
    if let Some(marker) = word.tone.and_then(tone_marker) {
        out.push(marker as char);
    }
    out.push_str(&letters[word.spans.coda.clone()]);
}

/// The parsed structure of a Vietnamese syllable over the original input text.
///
/// Offsets are expressed in characters of the analyzed `&str`. The internal
/// pipeline operates on canonical ASCII; this compatibility view maps results
/// back to the caller's text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyllableAnalysis {
    pub state: SequenceState,
    pub onset: Range<usize>,
    pub vowels: Range<usize>,
    pub coda: Range<usize>,
    pub main_vowel: Option<usize>,
    pub tone: Option<Tone>,
}

/// Analyzes one Unicode-scalar sequence as a Vietnamese syllable.
pub fn analyze_syllable(text: &str) -> SyllableAnalysis {
    analyze_syllable_with_orthography(text, Orthography::Modern)
}

pub fn analyze_syllable_with_orthography(text: &str, orthography: Orthography) -> SyllableAnalysis {
    let mut decoder = Decoder::new();
    let mut letters = String::with_capacity(text.len());
    // Per-byte mapping from the canonical buffer back to input chars.
    let mut map: Vec<u32> = Vec::with_capacity(text.len());
    for (char_index, character) in text.chars().enumerate() {
        let before = letters.len();
        decoder.decode(character, &mut letters);
        map.extend(std::iter::repeat_n(
            char_index as u32,
            letters.len() - before,
        ));
    }
    decoder.flush(&mut letters);

    let char_length = text.chars().count();
    let word = parse(&letters, orthography);
    if word.state == SequenceState::Invalid {
        return SyllableAnalysis {
            state: SequenceState::Invalid,
            onset: 0..0,
            vowels: 0..0,
            coda: char_length..char_length,
            main_vowel: None,
            tone: None,
        };
    }

    let at = |byte: usize| map[byte.min(map.len() - 1)] as usize;
    let span = |range: &Range<usize>| {
        if range.is_empty() {
            let edge = at(range.start);
            edge..edge
        } else {
            at(range.start)..at(range.end - 1) + 1
        }
    };
    SyllableAnalysis {
        state: word.state,
        onset: span(&word.spans.onset),
        vowels: span(&word.spans.vowels),
        coda: span(&word.spans.coda),
        main_vowel: word.main_vowel.map(|atom| {
            let grammar = rule(word.vowel);
            let mut byte = word.spans.vowels.start;
            for index in 0..atom as usize {
                byte += atom_width(&grammar.atoms, index);
            }
            at(byte)
        }),
        tone: word.tone,
    }
}
