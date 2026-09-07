use vietnamese_engine::{decode_vowel, encode_vowel, BaseVowel, Case, Tone};

#[test]
fn exhaustive_decode_consistency() {
    let mut decoded_count = 0usize;

    for cp in 0..=0x10FFFFu32 {
        let ch = match char::from_u32(cp) {
            Some(c) => c,
            None => {
                assert_eq!(decode_vowel('\0'), None);
                continue;
            }
        };

        if let Some((base, tone, case)) = decode_vowel(ch) {
            decoded_count += 1;
            assert_eq!(
                encode_vowel(base, tone, case),
                ch,
                "round-trip failed for U+{cp:04X} ('{ch}')"
            );
        }
    }

    // Chars that MUST decode: every Vietnamese vowel
    let lower = [
        'ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ', 'ê', 'ế', 'ề', 'ể', 'ễ', 'ệ', 'ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ',
        'ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ', 'â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ', 'ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự',
        'a', 'á', 'à', 'ả', 'ã', 'ạ', 'o', 'ó', 'ò', 'ỏ', 'õ', 'ọ', 'e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ',
        'i', 'í', 'ì', 'ỉ', 'ĩ', 'ị', 'u', 'ú', 'ù', 'ủ', 'ũ', 'ụ', 'y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ',
    ];
    let upper: Vec<char> = lower
        .iter()
        .map(|c| c.to_uppercase().next().unwrap())
        .collect();
    for c in lower.iter().chain(upper.iter()) {
        assert!(decode_vowel(*c).is_some(), "'{c}' must decode");
    }

    // Chars that MUST NOT decode
    for c in ['q', 'w', 'x', 'z', 'đ', 'Đ', '1', '!', ' ', 'å'] {
        assert_eq!(decode_vowel(c), None, "'{c}' must NOT decode");
    }

    // Decode must cover exactly the 144-chars table (72 lower + 72 upper)
    assert_eq!(
        decoded_count, 144,
        "unexpected decode coverage: {decoded_count}"
    );
}

#[test]
fn decode_all_precomposed_chars() {
    // Full expected set from ENCODED_VOWELS layout: base x tone x case
    let mut seen = 0;
    for id in 0..12usize {
        let base = BaseVowel::from_id(id);
        for tone_idx in 0..6usize {
            let tone = Tone::from_id(tone_idx);
            for case in [Case::Lower, Case::Upper] {
                let ch = encode_vowel(base, tone, case);
                assert_eq!(
                    decode_vowel(ch),
                    Some((base, tone, case)),
                    "decode('{ch}') mismatch for {base:?} {tone:?} {case:?}"
                );
                seen += 1;
            }
        }
    }
    assert_eq!(seen, 144);
}
