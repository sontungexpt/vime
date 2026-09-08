#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMethod {
    Telex,
    Vni,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestCase {
    method: InputMethod,
    word: &'static str,
    input: String,
    tone_position: usize,
    shape_position: usize,
}

const SEED_WORDS: [&str; 100] = [
    "ba", "ban", "banc", "bao", "bat", "bay", "be", "ben", "bi", "bin", "bo", "bon",
    "bong", "bu", "bua", "buoc", "buoi", "buon", "ca", "can", "cang", "cao", "cay",
    "cha", "chau", "chi", "chien", "chieu", "cho", "chu", "co", "con", "cong", "cu",
    "cua", "cuoi", "da", "dai", "dan", "dang", "dau", "day", "de", "den", "di", "dich",
    "do", "doi", "don", "dong", "du", "dua", "duoc", "duong", "em", "en", "gan", "ghe",
    "ghi", "gio", "giua", "ha", "hai", "han", "hang", "hao", "hay", "he", "hen", "hi",
    "hoa", "hoan", "hoc", "hoi", "hom", "hon", "huong", "ke", "keo", "khi", "kho",
    "la", "lai", "lam", "lan", "lang", "le", "len", "ly", "ma", "mai", "man", "mang",
    "me", "mien", "minh", "mo", "moi", "muon", "nha",
];

const TELEX_TONES: [char; 5] = ['s', 'f', 'r', 'x', 'j'];
const VNI_TONES: [char; 5] = ['1', '2', '3', '4', '5'];
const TELEX_SHAPES: [char; 5] = ['a', 'w', 'e', 'o', 'w'];
const VNI_SHAPES: [char; 5] = ['6', '7', '6', '7', '8'];
const VARIANTS_PER_WORD: usize = 50;

fn build_cases(method: InputMethod) -> Vec<TestCase> {
    let tone_keys = match method {
        InputMethod::Telex => &TELEX_TONES,
        InputMethod::Vni => &VNI_TONES,
    };
    let shape_keys = match method {
        InputMethod::Telex => &TELEX_SHAPES,
        InputMethod::Vni => &VNI_SHAPES,
    };

    let mut cases = Vec::with_capacity(SEED_WORDS.len() * VARIANTS_PER_WORD);

    for (word_index, &word) in SEED_WORDS.iter().enumerate() {
        let character_count = word.chars().count();

        for variant in 0..VARIANTS_PER_WORD {
            let tone_position =
                (word_index + variant) % (character_count + 1);
            let shape_position =
                (word_index * 3 + variant * 2 + 1) % (character_count + 1);
            let tone_key = tone_keys[variant % tone_keys.len()];
            let shape_key = shape_keys[variant % shape_keys.len()];
            let mut input =
                String::with_capacity(word.len() + tone_key.len_utf8() + shape_key.len_utf8());

            for (index, character) in word.chars().enumerate() {
                if index == tone_position {
                    input.push(tone_key);
                }
                if index == shape_position {
                    input.push(shape_key);
                }
                input.push(character);
            }
            if tone_position == character_count {
                input.push(tone_key);
            }
            if shape_position == character_count {
                input.push(shape_key);
            }

            cases.push(TestCase {
                method,
                word,
                input,
                tone_position,
                shape_position,
            });
        }
    }

    cases
}

#[test]
fn prepares_5000_telex_and_5000_vni_cases() {
    let telex_cases = build_cases(InputMethod::Telex);
    let vni_cases = build_cases(InputMethod::Vni);

    assert_eq!(telex_cases.len(), 5_000);
    assert_eq!(vni_cases.len(), 5_000);
    assert_eq!(telex_cases.len() + vni_cases.len(), 10_000);

    for case in telex_cases.iter().chain(vni_cases.iter()) {
        assert_eq!(case.word.chars().count(), case.input.chars().count() - 2);
        assert!(case.tone_position <= case.word.chars().count());
        assert!(case.shape_position <= case.word.chars().count());
        assert!(matches!(case.method, InputMethod::Telex | InputMethod::Vni));
    }
}
