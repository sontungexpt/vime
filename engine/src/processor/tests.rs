use super::{
    analyze_syllable, analyze_syllable_with_orthography, Orthography, Processor, SequenceState,
};
use crate::character::{
    decode_vowel, encode_vowel, BaseVowel, Case, RootVowel, Shape, Tone, Vowel,
};
use crate::Operation;

#[test]
fn semantic_vowel_transform_flow() {
    // a → (circumflex) → â → (acute) → ấ
    let plain = Vowel::new(BaseVowel::A, Tone::Flat, Case::Lower);
    let processor = Processor::new();
    let circumflexed = processor
        .apply(plain, Operation::Shapeable('a', Shape::Circumflex))
        .unwrap();
    let toned = processor
        .apply(circumflexed, Operation::Toneable('s', Tone::Acute))
        .unwrap();

    assert_eq!(
        toned,
        Vowel::new(BaseVowel::ACircumflex, Tone::Acute, Case::Lower)
    );
    assert_eq!(
        encode_vowel(BaseVowel::ACircumflex, Tone::Acute, Case::Lower),
        'ấ'
    );
    assert_eq!(
        decode_vowel('ấ'),
        Some((BaseVowel::ACircumflex, Tone::Acute, Case::Lower))
    );

    // Tone(Flat) undoes only the tone.
    let reverted = processor
        .apply(toned, Operation::Toneable('z', Tone::Flat))
        .unwrap();
    assert_eq!(
        encode_vowel(BaseVowel::ACircumflex, Tone::Flat, Case::Lower),
        'â'
    );
    assert_eq!(
        decode_vowel('â'),
        Some((BaseVowel::ACircumflex, Tone::Flat, Case::Lower))
    );
    assert_eq!(reverted.tone(), Tone::Flat);

    // Insertions, deletions, and consonant strokes do not apply to vowels.
    assert_eq!(processor.apply(plain, Operation::Insert('x')), None);
    assert_eq!(
        processor.apply(plain, Operation::Shapeable('d', Shape::Stroke)),
        None
    );
}

#[test]
fn codec_exposes_the_typed_shape_family() {
    assert_eq!(
        decode_vowel('ấ').map(|(b, _, _)| b.root()),
        Some(RootVowel::A)
    );
    assert_eq!(
        decode_vowel('Ắ').map(|(b, _, _)| b.shape()),
        Some(Shape::Breve)
    );
}

#[test]
fn returns_structure_and_main_vowel() {
    let analysis = analyze_syllable("nguyễn");
    assert_eq!(analysis.state, SequenceState::Valid);
    assert_eq!(analysis.onset, 0..2);
    assert_eq!(analysis.vowels, 2..5);
    assert_eq!(analysis.coda, 5..6);
    assert_eq!(analysis.main_vowel, Some(4));
    assert_eq!(analysis.tone, Some(Tone::Tilde));
}

#[test]
fn selects_main_vowel_using_requested_orthography() {
    assert_eq!(analyze_syllable("hoa").main_vowel, Some(2));
    assert_eq!(
        analyze_syllable_with_orthography("hoa", Orthography::Old).main_vowel,
        Some(1)
    );
}

#[test]
fn distinguishes_transitional_vowels() {
    // "uo" and "ue" only become words once a coda or horn completes them.
    assert_eq!(analyze_syllable("truong").state, SequenceState::Valid);
    assert_eq!(analyze_syllable("nguoi").state, SequenceState::Transitional);
    assert_eq!(analyze_syllable("hue").state, SequenceState::Transitional);
}

#[test]
fn validates_onset_and_coda() {
    assert_eq!(analyze_syllable("banhch").state, SequenceState::Invalid);
    assert_eq!(analyze_syllable("trang").state, SequenceState::Valid);
    assert_eq!(analyze_syllable("qwe").state, SequenceState::Invalid);
}

#[test]
fn arbitrary_unicode_is_safe() {
    let analysis = analyze_syllable("日本語🙂");
    assert_eq!(analysis.state, SequenceState::Invalid);
}
