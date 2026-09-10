use vietnamese_engine::{
    phonology::{Coda, Onset},
    BaseVowel, BufferChar, Case, DefaultKeyMapping, ParsedResult, Parser, Syllable, Tone,
};

fn parse_telex(input: &[BufferChar]) -> ParsedResult {
    Parser::parse(input, &DefaultKeyMapping::telex())
}

fn parse_vni(input: &[BufferChar]) -> ParsedResult {
    Parser::parse(input, &DefaultKeyMapping::vni())
}

fn parse_viqr(input: &[BufferChar]) -> ParsedResult {
    Parser::parse(input, &DefaultKeyMapping::viqr())
}

fn lit(word: &str) -> Vec<BufferChar> {
    word.chars().map(BufferChar::Literal).collect()
}

/// `word` followed by the given transform keys (tone and/or shape).
fn keys(word: &str, transforms: &[char]) -> Vec<BufferChar> {
    let mut seq = lit(word);
    seq.extend(transforms.iter().copied().map(BufferChar::Transform));
    seq
}

fn assert_ok(result: ParsedResult) -> Syllable {
    match result {
        ParsedResult::Success(s) => s,
        other => panic!("expected Success, got {other:?}"),
    }
}

fn assert_only_d_stroke(result: ParsedResult) {
    match result {
        ParsedResult::OnlyDAndStroke => {}
        other => panic!("expected OnlyDAndStroke, got {other:?}"),
    }
}

fn bases(s: &Syllable) -> Vec<BaseVowel> {
    s.vowels.iter().map(|v| v.base).collect()
}

fn cases(s: &Syllable) -> Vec<Case> {
    s.vowels.iter().map(|v| v.case).collect()
}

// =========================================================================
// TELEX SHAPE KEYS
// `w` is the only telex shape key reachable as a Transform: breve on `a`,
// horn on `o`/`u`. `x` is a *tone* key (tilde), so it never applies a shape.
// =========================================================================

#[test]
fn telex_w_breve_on_a() {
    let s = assert_ok(parse_telex(&keys("a", &['w'])));
    assert_eq!(s.tone, Tone::Flat);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);
    assert_eq!(s.onset.kind, Onset::None);
    assert_eq!(s.coda.kind, Coda::None);
}

#[test]
fn telex_w_horn_on_o() {
    let s = assert_ok(parse_telex(&keys("o", &['w'])));
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);
}

#[test]
fn telex_w_horn_on_u() {
    let s = assert_ok(parse_telex(&keys("u", &['w'])));
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn telex_x_is_tilde_not_circumflex() {
    // `x` is configured as a tone key in telex, so it yields Tilde and never
    // applies the circumflex shape that `x` also maps to in the config.
    let s = assert_ok(parse_telex(&keys("a", &['x'])));
    assert_eq!(s.tone, Tone::Tilde);
    assert_eq!(bases(&s), vec![BaseVowel::A]);
}

#[test]
fn telex_aa_and_oo_are_dead_sequences() {
    // Repeated letters form an invalid (Dead) vowel sequence -> whole parse fails.
    let result = parse_telex(&lit("aa"));
    match result {
        ParsedResult::Failure(_, msg) => assert_eq!(msg, "Dead sequence"),
        other => panic!("expected Failure(Dead sequence), got {other:?}"),
    }
    let result = parse_telex(&lit("oo"));
    match result {
        ParsedResult::Failure(_, msg) => assert_eq!(msg, "Dead sequence"),
        other => panic!("expected Failure(Dead sequence), got {other:?}"),
    }
}

// =========================================================================
// TELEX SHAPE + TONE
// =========================================================================

#[test]
fn telex_w_then_tone() {
    let s = assert_ok(parse_telex(&keys("a", &['w', 's'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_telex(&keys("o", &['w', 's'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);

    let s = assert_ok(parse_telex(&keys("u", &['w', 'j'])));
    assert_eq!(s.tone, Tone::Dot);
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn telex_shape_with_onset() {
    let s = assert_ok(parse_telex(&keys("da", &['w'])));
    assert_eq!(s.onset.kind, Onset::D);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_telex(&keys("mo", &['w'])));
    assert_eq!(s.onset.kind, Onset::M);
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);

    let s = assert_ok(parse_telex(&keys("thu", &['w'])));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn telex_shape_with_coda() {
    let s = assert_ok(parse_telex(&keys("an", &['w'])));
    assert_eq!(s.coda.kind, Coda::N);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);
}

#[test]
fn telex_shape_on_multi_vowel_real_word() {
    // "thương" = th + u + ơ (horn lands on o of the uo pair) + ng + acute
    let s = assert_ok(parse_telex(&keys("thuong", &['w', 's'])));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::U, BaseVowel::OHorn]);
    assert_eq!(s.coda.kind, Coda::Ng);
}

#[test]
fn telex_shape_preserves_upper_case() {
    let s = assert_ok(parse_telex(&keys("A", &['w'])));
    assert_eq!(cases(&s), vec![Case::Upper]);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);
}

#[test]
fn telex_shape_never_applied_to_invalid_sequence() {
    // "ăo" / "aơ" are not valid vowel sequences, so the shape is rejected.
    let s = assert_ok(parse_telex(&keys("ao", &['w'])));
    assert_eq!(bases(&s), vec![BaseVowel::A, BaseVowel::O]);
}

// =========================================================================
// TELEX D STROKE
// =========================================================================

#[test]
fn telex_d_stroke() {
    assert_only_d_stroke(parse_telex(&keys("d", &['d'])));
}

// =========================================================================
// VNI SHAPE KEYS
// `6` = circumflex (a/e/o), `7` = breve (a) or horn (o), `8` = horn (u),
// `9` = stroke (d). Digits `1`..`5` are tone keys.
// =========================================================================

#[test]
fn vni_6_circumflex() {
    let s = assert_ok(parse_vni(&keys("a", &['6'])));
    assert_eq!(s.tone, Tone::Flat);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);

    let s = assert_ok(parse_vni(&keys("e", &['6'])));
    assert_eq!(bases(&s), vec![BaseVowel::ECircumflex]);

    let s = assert_ok(parse_vni(&keys("o", &['6'])));
    assert_eq!(bases(&s), vec![BaseVowel::OCircumflex]);
}

#[test]
fn vni_7_breve_and_horn() {
    let s = assert_ok(parse_vni(&keys("a", &['7'])));
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_vni(&keys("o", &['7'])));
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);
}

#[test]
fn vni_8_horn_on_u() {
    let s = assert_ok(parse_vni(&keys("u", &['8'])));
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn vni_shape_then_tone() {
    let s = assert_ok(parse_vni(&keys("a", &['6', '1'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);

    let s = assert_ok(parse_vni(&keys("o", &['7', '1'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);

    let s = assert_ok(parse_vni(&keys("u", &['8', '5'])));
    assert_eq!(s.tone, Tone::Dot);
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn vni_tone_then_shape() {
    let s = assert_ok(parse_vni(&keys("a", &['1', '6'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);
}

#[test]
fn vni_shape_with_onset_and_coda() {
    let s = assert_ok(parse_vni(&keys("da", &['7'])));
    assert_eq!(s.onset.kind, Onset::D);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_vni(&keys("an", &['7'])));
    assert_eq!(s.coda.kind, Coda::N);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_vni(&keys("thanh", &['7'])));
    assert_eq!(s.onset.kind, Onset::Th);
    assert_eq!(s.coda.kind, Coda::Nh);
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);
}

#[test]
fn vni_shape_replacement() {
    // `a` + `7` = ă, then `6` replaces the breve with a circumflex → â.
    let s = assert_ok(parse_vni(&keys("a", &['7', '6'])));
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);
}

#[test]
fn vni_shape_preserves_upper_case() {
    let s = assert_ok(parse_vni(&keys("A", &['6'])));
    assert_eq!(cases(&s), vec![Case::Upper]);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);
}

#[test]
fn vni_shape_never_applied_to_invalid_sequence() {
    // "ăi" is not a valid sequence, so the shape is rejected.
    let s = assert_ok(parse_vni(&keys("ai", &['6'])));
    assert_eq!(bases(&s), vec![BaseVowel::A, BaseVowel::I]);
}

#[test]
fn vni_d_stroke() {
    assert_only_d_stroke(parse_vni(&keys("d", &['9'])));
}

// =========================================================================
// VIQR SHAPE KEYS
// `^` = circumflex (a/e/o), `(` = breve (a), `+` = horn (o/u),
// `d` = stroke (d). `` ` ``, `'`, `?`, `~`, `.` are tone keys.
// =========================================================================

#[test]
fn viqr_circumflex() {
    let s = assert_ok(parse_viqr(&keys("a", &['^'])));
    assert_eq!(s.tone, Tone::Flat);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);

    let s = assert_ok(parse_viqr(&keys("e", &['^'])));
    assert_eq!(bases(&s), vec![BaseVowel::ECircumflex]);

    let s = assert_ok(parse_viqr(&keys("o", &['^'])));
    assert_eq!(bases(&s), vec![BaseVowel::OCircumflex]);
}

#[test]
fn viqr_breve_and_horn() {
    let s = assert_ok(parse_viqr(&keys("a", &['('])));
    assert_eq!(bases(&s), vec![BaseVowel::ABreve]);

    let s = assert_ok(parse_viqr(&keys("o", &['+'])));
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);

    let s = assert_ok(parse_viqr(&keys("u", &['+'])));
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn viqr_shape_and_tone_order() {
    let s = assert_ok(parse_viqr(&keys("a", &['^', '\''])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);

    let s = assert_ok(parse_viqr(&keys("a", &['\'', '^'])));
    assert_eq!(s.tone, Tone::Acute);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);

    let s = assert_ok(parse_viqr(&keys("o", &['+', '`'])));
    assert_eq!(s.tone, Tone::Grave);
    assert_eq!(bases(&s), vec![BaseVowel::OHorn]);
}

#[test]
fn viqr_shape_with_coda() {
    let s = assert_ok(parse_viqr(&keys("an", &['^'])));
    assert_eq!(s.coda.kind, Coda::N);
    assert_eq!(bases(&s), vec![BaseVowel::ACircumflex]);
}

#[test]
fn viqr_shape_with_onset() {
    let s = assert_ok(parse_viqr(&keys("du", &['+'])));
    assert_eq!(s.onset.kind, Onset::D);
    assert_eq!(bases(&s), vec![BaseVowel::UHorn]);
}

#[test]
fn viqr_d_stroke() {
    assert_only_d_stroke(parse_viqr(&keys("d", &['d'])));
    assert_only_d_stroke(parse_viqr(&keys("D", &['d'])));
}
