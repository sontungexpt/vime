//! Tone markers and tone placement rules.

use super::rules::{
    Coda, VowelRule, MAX_ATOMS, Q_ABREVE, Q_ACIRC, Q_ECIRC, Q_OCIRC, Q_OHORN, Q_UHORN,
};
use crate::character::Tone;

/// Vietnamese orthography mode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Orthography {
    #[default]
    Modern,
    Old,
}

/// Whether `byte` is a tone-marker letter. `z` cancels the tone.
pub(crate) fn is_marker_byte(byte: u8) -> bool {
    matches!(byte, b's' | b'f' | b'r' | b'x' | b'j' | b'z')
}

/// The tone a marker letter produces; `z` cancels to no tone.
fn marker_tone(byte: u8) -> Option<Tone> {
    match byte {
        b's' => Some(Tone::Acute),
        b'f' => Some(Tone::Grave),
        b'r' => Some(Tone::Hook),
        b'x' => Some(Tone::Tilde),
        b'j' => Some(Tone::Dot),
        _ => None,
    }
}

/// ASCII marker byte for a tone, if it has a single-character marker.
pub(crate) fn tone_marker(tone: Tone) -> Option<u8> {
    match tone {
        Tone::Acute => Some(b's'),
        Tone::Grave => Some(b'f'),
        Tone::Hook => Some(b'r'),
        Tone::Tilde => Some(b'x'),
        Tone::Dot => Some(b'j'),
        Tone::Flat => None,
    }
}

/// Numeric tone index for the codec (`Flat` → 0, ... `Dot` → 5).
pub(crate) fn tone_index(tone: Tone) -> usize {
    tone as usize
}

/// Consumes a run of tone-marker bytes, last one winning; `z` cancels.
pub(crate) fn read_tones(bytes: &[u8], slot: &mut usize) -> Option<Tone> {
    let mut tone = None;
    while *slot < bytes.len() && is_marker_byte(bytes[*slot]) {
        tone = marker_tone(bytes[*slot]);
        *slot += 1;
    }
    tone
}

/// Relocates a tone marker typed after the coda (`vangf` → grave on `a`).
pub(crate) fn rescue_trailing_tone(
    bytes: &[u8],
    coda_start: usize,
) -> Option<(usize, Option<Tone>)> {
    let mut end = bytes.len();
    while end > coda_start && is_marker_byte(bytes[end - 1]) {
        end -= 1;
    }
    if end == bytes.len() || Coda::identify(&bytes[coda_start..end]).is_none() {
        return None;
    }
    let mut slot = end;
    Some((end, read_tones(bytes, &mut slot)))
}

/// Main-vowel atom index for tone placement.
pub(crate) fn main_index(grammar: &VowelRule, orthography: Orthography, has_coda: bool) -> u8 {
    if orthography == Orthography::Old {
        return old_main_index(grammar.count, grammar.atoms, has_coda);
    }
    if grammar.main != super::rules::MAIN_NONE {
        return grammar.main;
    }
    priority_main_index(grammar.count, grammar.atoms)
}

/// Old-orthography heuristic preserved from the previous engine.
fn old_main_index(count: u8, atoms: [u8; MAX_ATOMS], has_coda: bool) -> u8 {
    let mut index = usize::from(count == 3 || has_coda);
    for (candidate, &quality) in atoms.iter().enumerate().take(count as usize) {
        if matches!(
            quality,
            Q_ABREVE | Q_ACIRC | Q_ECIRC | Q_OCIRC | Q_OHORN | Q_UHORN
        ) {
            index = candidate;
        }
    }
    index as u8
}

/// Quality-priority fallback for sequences without explicit metadata.
fn priority_main_index(count: u8, atoms: [u8; MAX_ATOMS]) -> u8 {
    // Ranks per quality: ơ ê ă ô â ư a o e i u y.
    const RANKS: [u8; 12] = [6, 2, 4, 8, 1, 9, 7, 3, 0, 10, 5, 11];
    let mut best = (u8::MAX, 0u8);
    for (index, &quality) in atoms.iter().enumerate().take(count as usize) {
        let rank = RANKS[quality as usize];
        if rank < best.0 {
            best = (rank, index as u8);
        }
    }
    best.1
}
