use super::*;
use crate::keymapping::DefaultKeyMapping;

static TELEX: DefaultKeyMapping<'static> = DefaultKeyMapping::telex();

fn parser() -> Parser<'static, DefaultKeyMapping<'static>> {
    Parser::new(&TELEX)
}

#[test]
fn onset_b() {
    let mut p = parser();

    p.push(BufferChar::Literal('b'));

    assert_eq!(p.syllable.onset, None);
    assert_eq!(p.syllable.onset_chars, vec!['b']);
    assert_eq!(p.phase, ParsePhase::Onset);
}

#[test]
fn onset_ch() {
    let mut p = parser();

    p.push(BufferChar::Literal('c'));
    p.push(BufferChar::Literal('h'));

    assert_eq!(p.syllable.onset_chars, vec!['c', 'h']);
    assert_eq!(p.phase, ParsePhase::Onset);

    // Khi gặp vowel mới xác nhận onset.
    p.push(BufferChar::Literal('a'));

    assert_eq!(p.syllable.onset, Some(Onset::Ch));
    assert_eq!(p.syllable.vowels.len(), 1);
    assert_eq!(p.phase, ParsePhase::Vowel);
}

#[test]
fn onset_invalid() {
    let mut p = parser();

    p.push(BufferChar::Literal('b'));
    p.push(BufferChar::Literal('c'));
    let status = p.push(BufferChar::Literal('a'));

    assert_eq!(status, ParseStatus::Dead(DeadReason::InvalidOnset));
}

#[test]
fn onset_qu() {
    let mut p = parser();

    p.push(BufferChar::Literal('q'));
    p.push(BufferChar::Literal('u'));

    // "qu" vẫn đang được giữ trong onset.
    assert_eq!(p.syllable.onset_chars, vec!['q', 'u']);
    assert_eq!(p.phase, ParsePhase::Onset);

    p.push(BufferChar::Literal('a'));

    assert_eq!(p.syllable.onset, Some(Onset::QU));
    assert_eq!(p.syllable.vowels.len(), 1);
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    assert_eq!(p.phase, ParsePhase::Vowel);
}

#[test]
fn onset_gi() {
    let mut p = parser();

    p.push(BufferChar::Literal('g'));
    p.push(BufferChar::Literal('i'));

    // Chưa gặp vowel thứ hai nên vẫn chưa promote "gi".
    assert_eq!(p.syllable.onset, Some(Onset::G));
    assert_eq!(p.syllable.onset_chars, vec!['g']);
    assert_eq!(p.syllable.vowels.len(), 1);
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::I);

    // i + a => promote i thành onset.
    p.push(BufferChar::Literal('a'));

    assert_eq!(p.syllable.onset, Some(Onset::Gi));
    assert_eq!(p.syllable.onset_chars, vec!['g', 'i']);
    assert_eq!(p.syllable.vowels.len(), 1);
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
}

#[test]
fn tone_acute() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('s'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    assert_eq!(p.syllable.tone, Tone::Acute);
}

#[test]
fn tone_toggle_acute() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('s'));
    p.push(BufferChar::Transform('s'));

    // a + s -> á
    // á + s -> a + s(literal)
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    assert_eq!(p.syllable.tone, Tone::Flat);
}

#[test]
fn tone_replace() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('s')); // á
    p.push(BufferChar::Transform('f')); // à

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    assert_eq!(p.syllable.tone, Tone::Grave);
}

#[test]
fn shape_breve() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::ABreve);
}

#[test]
fn shape_toggle_breve() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('w'));
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
}

#[test]
fn tone_preserved_when_shape_added() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('s')); // á
    p.push(BufferChar::Transform('w')); // ắ

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::ABreve);
    assert_eq!(p.syllable.tone, Tone::Acute);
}

#[test]
fn tone_preserved_when_shape_removed() {
    let mut p = parser();

    p.push(BufferChar::Literal('a'));
    p.push(BufferChar::Transform('s')); // á
    p.push(BufferChar::Transform('w')); // ắ
    p.push(BufferChar::Transform('w')); // á + w literal

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::A);
    assert_eq!(p.syllable.tone, Tone::Acute);
}

#[test]
fn precomposed_vowel() {
    let mut p = parser();

    p.push(BufferChar::Literal('ắ'));

    assert_eq!(p.syllable.vowels.len(), 1);
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::ABreve);
    assert_eq!(p.syllable.tone, Tone::Acute);
    assert_eq!(p.syllable.vowels[0].case, Case::Lower);
}

#[test]
fn precomposed_vowel_then_change_tone() {
    let mut p = parser();

    p.push(BufferChar::Literal('ắ'));
    p.push(BufferChar::Transform('f'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::ABreve);
    assert_eq!(p.syllable.tone, Tone::Grave);
}

#[test]
fn precomposed_vowel_tone_toggle() {
    let mut p = parser();

    p.push(BufferChar::Literal('ắ'));
    p.push(BufferChar::Transform('s'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::ABreve);
    assert_eq!(p.syllable.tone, Tone::Flat);
}

#[test]
fn uo_reformats_to_uo_horn() {
    let mut p = parser();

    p.push(BufferChar::Literal('u'));
    p.push(BufferChar::Literal('o'));

    assert_eq!(p.syllable.vowels.len(), 2);
    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::O);
}

#[test]
fn uo_w_becomes_uo_horn() {
    let mut p = parser();

    p.push(BufferChar::Literal('u'));
    p.push(BufferChar::Literal('o'));
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OHorn);
}

#[test]
fn uo_circumflex() {
    let mut p = parser();

    p.push(BufferChar::Literal('u'));
    p.push(BufferChar::Literal('o'));
    p.push(BufferChar::Transform('o'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OCircumflex);
}

#[test]
fn uo_horn_then_circumflex() {
    let mut p = parser();

    p.push(BufferChar::Literal('u'));
    p.push(BufferChar::Literal('o'));
    p.push(BufferChar::Transform('w')); // uơ
    p.push(BufferChar::Transform('o')); // uô

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OCircumflex);
}

#[test]
fn u_horn_o_reformats_to_uo_horn() {
    let mut p = parser();

    p.push(BufferChar::Literal('ư'));
    p.push(BufferChar::Literal('o'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::UHorn);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OHorn);
}

#[test]
fn uo_horn_toggle() {
    let mut p = parser();

    p.push(BufferChar::Literal('u'));
    p.push(BufferChar::Literal('o'));

    // uo -> uơ
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OHorn);

    // uơ + w -> ươ
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::UHorn);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::OHorn);

    // ươ + w -> uow
    p.push(BufferChar::Transform('w'));

    assert_eq!(p.syllable.vowels[0].value, BaseVowel::U);
    assert_eq!(p.syllable.vowels[1].value, BaseVowel::O);
}
