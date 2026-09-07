//! Composition of parsed structures into Vietnamese text.

use super::rules::rule;
use super::syllable::{SequenceState, WordStructure};
use super::tone::tone_index;
use crate::character::codec::encode_vowel;
use crate::character::{BaseVowel, Case, Tone};

/// Renders a valid parsed word as precomposed Vietnamese text.
///
/// Transitional and invalid words render to an empty string; callers echo the
/// raw letters for those instead.
pub fn render_word(word: &WordStructure) -> String {
    if word.state != SequenceState::Valid {
        return String::new();
    }
    let mut out = String::with_capacity(16);
    out.push_str(word.onset.spelling());
    let grammar = rule(word.vowel);
    let tone = word.tone.map_or(0, tone_index);
    for index in 0..grammar.count as usize {
        let quality = grammar.atoms[index] as usize;
        let toned = word.main_vowel == Some(index as u8);
        out.push(encode_vowel(
            BaseVowel::from_id(quality),
            Tone::from_id(if toned { tone } else { 0 }),
            Case::Lower,
        ));
    }
    out.push_str(word.coda.spelling());
    out
}
