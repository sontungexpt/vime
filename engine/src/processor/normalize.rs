//! Canonicalization of arbitrary Unicode text into ASCII raw syllables.
//!
//! The decoder folds case, expands precomposed Vietnamese characters through
//! [`vietnamese_spelling`], and folds combining marks. Tones decoded mid-run
//! are parked until the vowel run ends so that `người` canonicalizes to
//! `nguowif` with its marker after the complete run.

use super::render_word;
use super::rules::vietnamese_spelling;
use super::syllable::{parse, serialize, SequenceState};
use super::tone::{is_marker_byte, Orthography};

#[inline]
fn is_vowel_letter(byte: u8) -> bool {
    matches!(byte, b'a' | b'e' | b'i' | b'o' | b'u' | b'y' | b'w')
}

/// Incremental decoder from Unicode characters to canonical ASCII letters.
pub(crate) struct Decoder {
    pending_tone: Option<u8>,
}

impl Decoder {
    pub(crate) fn new() -> Self {
        Self { pending_tone: None }
    }

    /// Decodes one character, appending canonical bytes to `out`.
    pub(crate) fn decode(&mut self, character: char, out: &mut String) {
        for lowered in character.to_lowercase() {
            self.push(lowered, out);
        }
    }

    /// Emits any parked tone; called when a vowel run ends.
    pub(crate) fn flush(&mut self, out: &mut String) {
        if let Some(tone) = self.pending_tone.take() {
            out.push(tone as char);
        }
    }

    fn push(&mut self, character: char, out: &mut String) {
        match character {
            '\u{0301}' => self.pending_tone = Some(b's'),
            '\u{0300}' => self.pending_tone = Some(b'f'),
            '\u{0309}' => self.pending_tone = Some(b'r'),
            '\u{0303}' => self.pending_tone = Some(b'x'),
            '\u{0323}' => self.pending_tone = Some(b'j'),
            // Breve and horn marks are folded into precomposed bodies.
            '\u{0306}' | '\u{031b}' => {}
            // Circumflex marks duplicate the previous base letter.
            '\u{0302}' => {
                if let Some(last) = out.chars().last() {
                    out.push(last);
                }
            }
            _ => {
                if let Some((body, tone)) = vietnamese_spelling(character) {
                    self.flush(out);
                    out.push_str(body);
                    self.pending_tone = tone;
                } else {
                    let mut buffer = [0u8; 4];
                    let encoded = character.encode_utf8(&mut buffer);
                    if !is_vowel_letter(encoded.as_bytes()[0]) {
                        self.flush(out);
                    }
                    out.push_str(encoded);
                }
            }
        }
    }
}

/// Appends the canonical ASCII form of `text` to `out`.
pub fn canonicalize_into(text: &str, out: &mut String) {
    let mut decoder = Decoder::new();
    for character in text.chars() {
        decoder.decode(character, out);
    }
    decoder.flush(out);
}

/// Lifts tone-marker letters stranded inside a vowel run to the run's end.
///
/// Typing a vowel after a committed marker (`nguowf` + `i`) leaves the
/// marker mid-run, which no longer parses. Moving every marker between the
/// first and last vowel letters to after the last vowel restores the
/// canonical order without touching consonant letters.
pub(crate) fn lift_markers(letters: &str) -> Option<String> {
    const VOWELS: &[u8] = b"aeiouyw";
    let bytes = letters.as_bytes();
    let first = bytes.iter().position(|byte| VOWELS.contains(byte))?;
    let last = bytes.iter().rposition(|byte| VOWELS.contains(byte))?;
    if first >= last {
        return None;
    }
    let mut repaired = String::with_capacity(bytes.len());
    let mut markers = String::new();
    for (index, &byte) in bytes.iter().enumerate() {
        if index > first && index < last && is_marker_byte(byte) {
            markers.push(byte as char);
        } else {
            repaired.push(byte as char);
        }
    }
    repaired.push_str(&markers);
    Some(repaired)
}

/// Renders a canonical raw buffer into Vietnamese text.
///
/// Valid syllables are composed (`"nguowif"` → `"người"`); anything else
/// passes through verbatim so the preview tracks the typed buffer.
///
/// The engine calls this whenever it needs the front-facing preview; the raw
/// buffer itself stays language-neutral.
pub fn render_raw(raw: &str, orthography: Orthography) -> String {
    let mut letters = String::with_capacity(raw.len());
    canonicalize_into(raw, &mut letters);
    let word = parse(&letters, orthography);
    if word.state == SequenceState::Valid {
        render_word(&word)
    } else {
        raw.to_string()
    }
}

/// Normalizes arbitrary text into canonical raw form.
///
/// Valid syllables are re-serialized canonically (merging duplicate horn
/// markers and relocating trailing tone markers); everything else passes
/// through verbatim.
pub fn normalize(text: &str) -> String {
    let mut letters = String::with_capacity(text.len());
    canonicalize_into(text, &mut letters);
    let word = parse(&letters, Orthography::Modern);
    let mut output = String::with_capacity(letters.len());
    if word.state == SequenceState::Valid {
        serialize(&word, &letters, &mut output);
        output
    } else {
        letters
    }
}
