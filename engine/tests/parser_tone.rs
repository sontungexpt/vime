use vietnamese_engine::{
    phonology::{encode_vowel, Coda, Onset},
    BaseVowel, BufferChar, Case, DefaultKeyMapping, ParsedResult, ParsedVowel, Parser, Syllable,
    Tone,
};

fn parse_telex(input: &[BufferChar]) -> ParsedResult {
    Parser::parse(input, &DefaultKeyMapping::telex())
}

fn parse_vni(input: &[BufferChar]) -> ParsedResult {
    Parser::parse(input, &DefaultKeyMapping::vni())
}

fn literal(word: &str) -> Vec<BufferChar> {
    word.chars().map(BufferChar::Literal).collect()
}

fn with_tone(word: &str, tone_key: char) -> Vec<BufferChar> {
    let mut chars = literal(word);
    chars.push(BufferChar::Transform(tone_key));
    chars
}

fn assert_ok(result: ParsedResult) -> Syllable {
    match result {
        ParsedResult::Success(s) => s,
        other => panic!("expected Success, got {other:?}"),
    }
}

fn assert_failure(result: ParsedResult, expected_msg: &str) {
    match result {
        ParsedResult::Failure(_, msg) => assert_eq!(msg, expected_msg),
        other => panic!("expected Failure({expected_msg:?}), got {other:?}"),
    }
}

fn flat_char(base: BaseVowel) -> char {
    encode_vowel(base, Tone::Flat, Case::Lower)
}

fn bases(vowels: &[ParsedVowel]) -> Vec<BaseVowel> {
    vowels.iter().map(|v| v.base).collect()
}

fn chars(vowels: &[ParsedVowel]) -> Vec<char> {
    vowels
        .iter()
        .map(|v| encode_vowel(v.base, Tone::Flat, v.case))
        .collect()
}

const TELEX_KEYS: [(char, Tone); 6] = [
    ('s', Tone::Acute),
    ('f', Tone::Grave),
    ('r', Tone::Hook),
    ('x', Tone::Tilde),
    ('j', Tone::Dot),
    ('z', Tone::Flat),
];

// The telex tone keys that actually produce a successful parse when applied to
// an already-flat syllable. `z` (Flat) is excluded: applying it to a flat
// syllable toggles the (already Flat) tone, then pushes the non-vowel key into
// the coda where it is invalid → `Failure("Invalid coda")`.
const TELEX_NONFLAT_KEYS: [(char, Tone); 5] = [
    ('s', Tone::Acute),
    ('f', Tone::Grave),
    ('r', Tone::Hook),
    ('x', Tone::Tilde),
    ('j', Tone::Dot),
];

// Same rationale for VNI: `0` (Flat) is excluded.
const VNI_NONFLAT_KEYS: [(char, Tone); 5] = [
    ('1', Tone::Acute),
    ('2', Tone::Grave),
    ('3', Tone::Hook),
    ('4', Tone::Tilde),
    ('5', Tone::Dot),
];

// =========================================================================
// 1. BASIC: telex tone keys on simple syllables
// =========================================================================

#[test]
fn telex_tone_keys_on_an() {
    for (key, expected_tone) in TELEX_NONFLAT_KEYS {
        let s = assert_ok(parse_telex(&with_tone("an", key)));
        assert_eq!(s.tone, expected_tone, "key {key:?}");
        assert_eq!(s.coda.collected, "n", "key {key:?}");
    }
}

#[test]
fn telex_tone_keys_on_ba() {
    for (key, expected_tone) in TELEX_NONFLAT_KEYS {
        let s = assert_ok(parse_telex(&with_tone("ba", key)));
        assert_eq!(s.tone, expected_tone, "key {key:?}");
        assert_eq!(s.onset.kind, Onset::B);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
        assert_eq!(chars(&s.vowels), vec!['a']);
        assert_eq!(s.coda.kind, Coda::None);
    }
}

#[test]
fn telex_tone_keys_on_to() {
    for (key, expected_tone) in TELEX_NONFLAT_KEYS {
        let s = assert_ok(parse_telex(&with_tone("to", key)));
        assert_eq!(s.tone, expected_tone, "key {key:?}");
        assert_eq!(s.onset.kind, Onset::T);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::O]);
        assert_eq!(chars(&s.vowels), vec!['o']);
        assert_eq!(s.coda.kind, Coda::None);
    }
}

#[test]
fn telex_tone_keys_on_aiueo() {
    for (word, expected_vowel) in [
        ("ta", BaseVowel::A),
        ("ti", BaseVowel::I),
        ("tu", BaseVowel::U),
        ("te", BaseVowel::E),
        ("to", BaseVowel::O),
    ] {
        let s = assert_ok(parse_telex(&with_tone(word, 's')));
        assert_eq!(s.tone, Tone::Acute, "word {word:?}");
        assert_eq!(s.onset.kind, Onset::T, "word {word:?}");
        assert_eq!(bases(&s.vowels), vec![expected_vowel], "word {word:?}");
        assert_eq!(s.coda.kind, Coda::None);
    }
}

#[test]
fn telex_y_is_treated_as_consonant_not_vowel() {
    // 'y' falls in the `'v'..='z'` range of `is_ascii_consonant`, so the parser
    // consumes "ty" as a two-character onset, which is not a valid onset.
    let result = parse_telex(&with_tone("ty", 's'));
    assert_failure(result, "Invalid onset");
}

// =========================================================================
// 2. CODA: tones with all coda types
// =========================================================================

#[test]
fn telex_tone_on_syllable_with_coda_n() {
    for (key, expected_tone) in [('s', Tone::Acute), ('f', Tone::Grave), ('j', Tone::Dot)] {
        let s = assert_ok(parse_telex(&with_tone("ban", key)));
        assert_eq!(s.tone, expected_tone, "key {key:?}");
        assert_eq!(s.onset.kind, Onset::B);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
        assert_eq!(s.coda.kind, Coda::N);
    }
}

#[test]
fn telex_tone_on_syllable_with_coda_t() {
    for (key, expected_tone) in [('s', Tone::Acute), ('f', Tone::Grave)] {
        let s = assert_ok(parse_telex(&with_tone("bat", key)));
        assert_eq!(s.tone, expected_tone);
        assert_eq!(s.onset.kind, Onset::B);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
        assert_eq!(s.coda.kind, Coda::T);
    }
}

#[test]
fn telex_tone_on_syllable_with_coda_c() {
    for (key, expected_tone) in [('s', Tone::Acute), ('r', Tone::Hook)] {
        let s = assert_ok(parse_telex(&with_tone("bac", key)));
        assert_eq!(s.tone, expected_tone);
        assert_eq!(s.coda.kind, Coda::C);
    }
}

#[test]
fn telex_tone_on_syllable_with_coda_p() {
    let s = assert_ok(parse_telex(&with_tone("bap", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.coda.kind, Coda::P);
}

#[test]
fn telex_tone_on_syllable_with_coda_m() {
    let s = assert_ok(parse_telex(&with_tone("bom", 'f')));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(s.coda.kind, Coda::M);
}

#[test]
fn telex_tone_on_syllable_with_coda_ng() {
    let s = assert_ok(parse_telex(&with_tone("bong", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.coda.kind, Coda::Ng);
    assert_eq!(s.coda.collected, "ng");
}

#[test]
fn telex_tone_on_syllable_with_coda_nh() {
    let s = assert_ok(parse_telex(&with_tone("banh", 'f')));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(s.coda.kind, Coda::Nh);
    assert_eq!(s.coda.collected, "nh");
}

#[test]
fn telex_tone_on_syllable_with_coda_ch() {
    let s = assert_ok(parse_telex(&with_tone("bach", 'x')));
    assert_eq!(s.tone, Tone::Tilde);
    assert_eq!(s.coda.kind, Coda::Ch);
}

// =========================================================================
// 3. COMPLEX ONSETS: tones with multi-character onsets
// =========================================================================

#[test]
fn telex_tone_on_th_onset() {
    let s = assert_ok(parse_telex(&with_tone("than", 's')));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    assert_eq!(s.coda.kind, Coda::N);
}

#[test]
fn telex_tone_on_ch_onset() {
    let s = assert_ok(parse_telex(&with_tone("chan", 'f')));
    assert_eq!(s.onset.kind, Onset::Ch);
    assert_eq!(s.tone, Tone::Grave);
}

#[test]
fn telex_tone_on_kh_onset() {
    let s = assert_ok(parse_telex(&with_tone("khan", 'r')));
    assert_eq!(s.onset.kind, Onset::Kh);
    assert_eq!(s.tone, Tone::Hook);
}

#[test]
fn telex_tone_on_ng_onset() {
    let s = assert_ok(parse_telex(&with_tone("ngan", 'x')));
    assert_eq!(s.onset.kind, Onset::Ng);
    assert_eq!(s.tone, Tone::Tilde);
}

#[test]
fn telex_tone_on_ngh_onset() {
    let s = assert_ok(parse_telex(&with_tone("nghan", 'j')));
    assert_eq!(s.onset.kind, Onset::Ngh);
    assert_eq!(s.tone, Tone::Dot);
}

#[test]
fn telex_tone_on_qu_onset() {
    let s = assert_ok(parse_telex(&with_tone("quan", 's')));
    assert_eq!(s.onset.kind, Onset::QU);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
}

#[test]
fn telex_tone_on_tr_onset() {
    let s = assert_ok(parse_telex(&with_tone("tran", 'f')));
    assert_eq!(s.onset.kind, Onset::Tr);
    assert_eq!(s.tone, Tone::Grave);
}

#[test]
fn telex_tone_on_gh_onset() {
    let s = assert_ok(parse_telex(&with_tone("ghan", 's')));
    assert_eq!(s.onset.kind, Onset::Gh);
    assert_eq!(s.tone, Tone::Acute);
}

#[test]
fn telex_tone_on_ph_onset() {
    let s = assert_ok(parse_telex(&with_tone("phin", 'f')));
    assert_eq!(s.onset.kind, Onset::Ph);
    assert_eq!(s.tone, Tone::Grave);
}

#[test]
fn telex_tone_on_gi_onset() {
    let s = assert_ok(parse_telex(&with_tone("gia", 's')));
    assert_eq!(s.onset.kind, Onset::Gi);
    assert_eq!(s.tone, Tone::Acute);
}

#[test]
fn telex_tone_on_d_stroke_onset() {
    let s = assert_ok(parse_telex(&with_tone("đan", 's')));
    assert_eq!(s.onset.kind, Onset::Đ);
    assert_eq!(s.tone, Tone::Acute);
}

#[test]
fn telex_tone_on_single_consonant_onsets() {
    let cases = [
        ("lan", Onset::L),
        ("man", Onset::M),
        ("nan", Onset::N),
        ("pan", Onset::P),
        ("ran", Onset::R),
        ("san", Onset::S),
        ("van", Onset::V),
        ("xan", Onset::X),
    ];

    for (word, expected_onset) in cases {
        let s = assert_ok(parse_telex(&with_tone(word, 's')));
        assert_eq!(s.onset.kind, expected_onset, "word {word:?}");
        assert_eq!(s.tone, Tone::Acute);
    }
}

// =========================================================================
// 4. VOWEL SEQUENCES: multiple vowels in a syllable
// =========================================================================

#[test]
fn telex_tone_on_ia_diphthong() {
    let s = assert_ok(parse_telex(&with_tone("tia", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::I, BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec!['i', 'a']);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn telex_tone_on_ua_diphthong() {
    let s = assert_ok(parse_telex(&with_tone("tua", 'f')));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec!['u', 'a']);
}

#[test]
fn telex_tone_on_ea_diphthong() {
    let s = assert_ok(parse_telex(&with_tone("tea", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::E, BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec!['e', 'a']);
}

#[test]
fn telex_tone_on_uy_diphthong() {
    // "tuy" → onset=T, vowels=[u, y] (both decode as vowels), coda=None
    let s = assert_ok(parse_telex(&with_tone("tuy", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::Y]);
    assert_eq!(chars(&s.vowels), vec!['u', 'y']);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn telex_tone_on_ui_diphthong() {
    // "tui" → onset=T, vowels=[u, i] (both decode as vowels), coda=None
    let s = assert_ok(parse_telex(&with_tone("tui", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::I]);
    assert_eq!(chars(&s.vowels), vec!['u', 'i']);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn telex_tone_on_uong_vowel_sequence() {
    // "tuong" → onset=T, vowels=[u, o], coda=ng
    let s = assert_ok(parse_telex(&with_tone("tuong", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::T);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::O]);
    assert_eq!(chars(&s.vowels), vec!['u', 'o']);
    assert_eq!(s.coda.kind, Coda::Ng);
}

#[test]
fn telex_tone_on_uon_vowel_sequence() {
    let s = assert_ok(parse_telex(&with_tone("tuon", 'r')));
    assert_eq!(s.tone, Tone::Hook);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::O]);
    assert_eq!(s.coda.kind, Coda::N);
}

#[test]
fn telex_tone_does_not_change_vowels_or_coda() {
    let s = assert_ok(parse_telex(&with_tone("uong", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(chars(&s.vowels), vec!['u', 'o']);
    assert_eq!(s.coda.collected, "ng");
}

// =========================================================================
// 5. VNI TONE KEYS
// =========================================================================

#[test]
fn vni_tone_keys_on_an() {
    for (key, expected_tone) in VNI_NONFLAT_KEYS {
        let s = assert_ok(parse_vni(&with_tone("an", key)));
        assert_eq!(s.tone, expected_tone, "key {key:?}");
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
        assert_eq!(chars(&s.vowels), vec!['a']);
    }
}

#[test]
fn vni_tone_keys_on_ba() {
    for (key, expected_tone) in VNI_NONFLAT_KEYS {
        let s = assert_ok(parse_vni(&with_tone("ba", key)));
        assert_eq!(s.tone, expected_tone);
        assert_eq!(s.onset.kind, Onset::B);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    }
}

#[test]
fn vni_tone_keys_on_to() {
    for (key, expected_tone) in VNI_NONFLAT_KEYS {
        let s = assert_ok(parse_vni(&with_tone("to", key)));
        assert_eq!(s.tone, expected_tone);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::O]);
    }
}

#[test]
fn vni_tone_on_coda() {
    let s = assert_ok(parse_vni(&with_tone("ban", '1')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.coda.kind, Coda::N);

    let s = assert_ok(parse_vni(&with_tone("bat", '2')));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(s.coda.kind, Coda::T);

    let s = assert_ok(parse_vni(&with_tone("bong", '3')));
    assert_eq!(s.tone, Tone::Hook);
    assert_eq!(s.coda.kind, Coda::Ng);
}

#[test]
fn vni_tone_on_complex_onset() {
    let s = assert_ok(parse_vni(&with_tone("than", '1')));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.tone, Tone::Acute);

    let s = assert_ok(parse_vni(&with_tone("ngan", '4')));
    assert_eq!(s.onset.kind, Onset::Ng);
    assert_eq!(s.tone, Tone::Tilde);
}

#[test]
fn vni_tone_on_vowel_sequence() {
    let s = assert_ok(parse_vni(&with_tone("tia", '1')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::I, BaseVowel::A]);

    let s = assert_ok(parse_vni(&with_tone("tuong", '2')));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::O]);
}

// =========================================================================
// 6. FLAT TONE KEY ON AN ALREADY-FLAT SYLLABLE
//
// `z` (telex) and `0` (vni) map to Tone::Flat. Applied to a syllable whose
// tone is already Flat, the toggle branch pushes the non-vowel key into the
// coda, where it is invalid → Failure("Invalid coda").
// =========================================================================

#[test]
fn telex_flat_key_on_flat_syllable_fails_invalid_coda() {
    for word in ["a", "an", "ba", "to"] {
        let result = parse_telex(&with_tone(word, 'z'));
        assert_failure(result, "Invalid coda");
    }
}

#[test]
fn vni_flat_key_on_flat_syllable_fails_invalid_coda() {
    for word in ["a", "an", "ba", "to"] {
        let result = parse_vni(&with_tone(word, '0'));
        assert_failure(result, "Invalid coda");
    }
}

// =========================================================================
// 7. TONE REPLACE (different tone key) & TOGGLE (same tone key)
// =========================================================================

#[test]
fn telex_tone_key_replaces_different_tone() {
    // "an" + 's' (Acute) then 'f' (Grave) → Grave replaces acute
    let input = vec![
        BufferChar::Literal('a'),
        BufferChar::Literal('n'),
        BufferChar::Transform('s'),
        BufferChar::Transform('f'),
    ];
    let s = assert_ok(parse_telex(&input));
    assert_eq!(s.tone, Tone::Grave);
}

#[test]
fn telex_tone_key_replaces_prewritten_tone() {
    // "bá" (Acute) + 'f' (Grave) → Grave replaces the prewritten tone
    let input = vec![
        BufferChar::Literal('b'),
        BufferChar::Literal('á'),
        BufferChar::Transform('f'),
    ];
    let s = assert_ok(parse_telex(&input));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(s.onset.kind, Onset::B);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
}

#[test]
fn telex_same_tone_key_after_prewritten_vowel_preserves_tone() {
    // "bá" (Acute) + 's' (Acute): the prewritten tone is preserved → Acute
    let input = vec![
        BufferChar::Literal('b'),
        BufferChar::Literal('á'),
        BufferChar::Transform('s'),
    ];
    let s = assert_ok(parse_telex(&input));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::B);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
}

#[test]
fn telex_same_tone_key_after_coda_fails_invalid_coda() {
    // "an" + 's' (Acute) then 's' (same) → the second 's' lands in the coda →
    // Failure("Invalid coda").
    let input = vec![
        BufferChar::Literal('a'),
        BufferChar::Literal('n'),
        BufferChar::Transform('s'),
        BufferChar::Transform('s'),
    ];
    assert_failure(parse_telex(&input), "Invalid coda");
}

#[test]
fn vni_same_tone_key_fails_invalid_coda() {
    let input = vec![
        BufferChar::Literal('a'),
        BufferChar::Literal('n'),
        BufferChar::Transform('1'),
        BufferChar::Transform('1'),
    ];
    assert_failure(parse_vni(&input), "Invalid coda");
}

// =========================================================================
// 8. TONE CONFLICT
//
// A conflict only occurs when a prewritten toned *Literal* vowel is followed
// by another prewritten toned Literal vowel in the vowel stream. Transform
// tone keys never conflict — they replace.
// =========================================================================

#[test]
fn tone_conflict_two_prewritten_toned_vowels() {
    // 'á' (Acute) + 'ằ' (Grave): both non-flat literals in sequence → conflict
    let input = vec![
        BufferChar::Literal('b'),
        BufferChar::Literal('á'),
        BufferChar::Literal('ằ'),
    ];
    assert_failure(parse_telex(&input), "Tone conflict");
}

#[test]
fn no_conflict_when_second_literal_is_flat() {
    // 'á' (Acute) + 'a' (Flat): the second literal is flat → no conflict
    let input = vec![
        BufferChar::Literal('b'),
        BufferChar::Literal('á'),
        BufferChar::Literal('a'),
    ];
    let s = assert_ok(parse_telex(&input));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A, BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec!['a', 'a']);
}

// =========================================================================
// 9. PRE-WRITTEN TONED CHARACTERS (literal toned vowels)
// =========================================================================

#[test]
fn prewritten_acute_vowel() {
    let s = assert_ok(parse_telex(&literal("bá")));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::B);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec![flat_char(BaseVowel::A)]);
}

#[test]
fn prewritten_grave_vowel() {
    let s = assert_ok(parse_telex(&literal("bà")));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
}

#[test]
fn prewritten_hook_vowel() {
    let s = assert_ok(parse_telex(&literal("bả")));
    assert_eq!(s.tone, Tone::Hook);
}

#[test]
fn prewritten_tilde_vowel() {
    let s = assert_ok(parse_telex(&literal("bã")));
    assert_eq!(s.tone, Tone::Tilde);
}

#[test]
fn prewritten_dot_vowel() {
    let s = assert_ok(parse_telex(&literal("bạ")));
    assert_eq!(s.tone, Tone::Dot);
}

#[test]
fn prewritten_tone_on_complex_vowel() {
    let s = assert_ok(parse_telex(&literal("ớ")));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::OHorn]);
}

#[test]
fn prewritten_tone_with_coda() {
    let s = assert_ok(parse_telex(&literal("án")));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::None);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    assert_eq!(s.coda.kind, Coda::N);
}

#[test]
fn prewritten_tone_with_onset_and_coda() {
    let s = assert_ok(parse_telex(&literal("thắng")));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::ABreve]);
    assert_eq!(s.coda.kind, Coda::Ng);
}

#[test]
fn prewritten_flat_vowel_has_flat_tone() {
    let s = assert_ok(parse_telex(&literal("ba")));
    assert_eq!(s.tone, Tone::Flat);
}

#[test]
fn uppercase_prewritten_tone() {
    let s = assert_ok(parse_telex(&literal("Bá")));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::B);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
}

// =========================================================================
// 10. UPPERCASE ONSET + TONE
// =========================================================================

#[test]
fn uppercase_onset_with_tone() {
    let cases = [
        ("Tan", Onset::T),
        ("BaN", Onset::B),
        ("Khan", Onset::Kh),
        ("NgAn", Onset::Ng),
        ("PHan", Onset::Ph),
    ];

    for (word, expected_onset) in cases {
        let s = assert_ok(parse_telex(&with_tone(word, 's')));
        assert_eq!(s.onset.kind, expected_onset, "word {word:?}");
        assert_eq!(s.tone, Tone::Acute);
    }
}

// =========================================================================
// 11. EDGE CASES
// =========================================================================

#[test]
fn no_onset_just_vowel_and_tone() {
    let s = assert_ok(parse_telex(&with_tone("a", 's')));
    assert_eq!(s.onset.kind, Onset::None);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn no_onset_vowel_and_coda_with_tone() {
    let s = assert_ok(parse_telex(&with_tone("an", 'f')));
    assert_eq!(s.onset.kind, Onset::None);
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(s.coda.kind, Coda::N);
}

#[test]
fn tone_on_long_vowel_sequence() {
    let s = assert_ok(parse_telex(&with_tone("tuong", 's')));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(s.onset.kind, Onset::T);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::U, BaseVowel::O]);
    assert_eq!(s.coda.kind, Coda::Ng);
}

#[test]
fn tone_on_single_vowel_no_coda_nonflat_tones() {
    for (key, expected) in TELEX_NONFLAT_KEYS {
        let s = assert_ok(parse_telex(&with_tone("a", key)));
        assert_eq!(s.tone, expected, "key {key:?}");
        assert_eq!(s.onset.kind, Onset::None);
        assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
        assert_eq!(s.coda.kind, Coda::None);
    }
}

#[test]
fn coda_none_just_vowel_with_tone() {
    let s = assert_ok(parse_telex(&with_tone("o", 'j')));
    assert_eq!(s.tone, Tone::Dot);
    assert_eq!(s.onset.kind, Onset::None);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::O]);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn full_syllable_with_all_parts_and_tone() {
    // "thanh" + 's' → onset=Th, vowels=[a], coda=nh, tone=Acute
    let s = assert_ok(parse_telex(&with_tone("thanh", 's')));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s.vowels), vec![BaseVowel::A]);
    assert_eq!(chars(&s.vowels), vec!['a']);
    assert_eq!(s.coda.kind, Coda::Nh);
    assert_eq!(s.coda.collected, "nh");
}
